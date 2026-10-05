//! Recorder repair Job tests (cancel, shutdown, and restore).

//! Native repair through the Harness (layer L3, paused clock).

mod common;

use core::{num::NonZeroU32, sync::atomic::Ordering, time::Duration};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Serialize;
use tempfile::tempdir;
use tokio::time::{advance, timeout};

use blueos_comms::{CommsBackend, Subscriber, channel::ChannelBackend};
use blueos_idl::Message;
use blueos_idl::msg::blueos_msgs::{CommandAckStatus, JobStatusStatus};
use blueos_idl::msg::blueos_recorder_msgs::{RepairRecordingGoal, RepairRecordingResult};
use blueos_jobs::{JobControl, JobId, JobNature, Jobs};
use blueos_recorder_app::RecorderService;
use blueos_recorder_domain::durable::RecorderDurableState;
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_service::{Service, ServiceContext, new_job_id, testing::Harness};
use blueos_settings::ServiceStateStore;

use common::harness::io::{drain_blocking_io, set_modified_seconds_ago};
use common::harness::jobs::{next_job_result, repair_job_status, subscribe_job_results};
use common::harness::startup::start_harness_with;
use common::harness::state::{
    wait_for_library_file_listed, wait_for_library_file_not_repairing, wait_for_library_state,
};
use common::mcap_fixtures::write_truncated_mcap;
use common::recorder_arguments;
use common::repair_rewrite::HeldRewrite;

#[derive(Serialize)]
struct PersistedRecorderState {
    #[serde(rename = "VERSION")]
    version: u32,
    domain: RecorderDurableState,
    jobs: Jobs,
}

struct CancelExpectation<'a> {
    harness: &'a Harness<RecorderService>,
    results: &'a mut Subscriber,
    job_id: JobId,
    relative_path: &'a str,
    path: &'a Path,
    original: &'a [u8],
    directory: &'a Path,
}

fn recover_files(directory: &Path) -> Vec<String> {
    fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.ends_with(".recover").then_some(name)
        })
        .collect()
}

async fn held_truncated_repair_harness(
    file_name: &str,
) -> (
    tempfile::TempDir,
    Harness<RecorderService>,
    HeldRewrite,
    PathBuf,
    Vec<u8>,
    Subscriber,
) {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join(file_name);
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);
    let original = fs::read(&path).expect("read");
    let held = HeldRewrite::new();
    let harness = start_harness_with(directory.path(), |context| held.wrap(context)).await;
    let results = subscribe_job_results(&harness, "RepairRecording").await;
    wait_for_library_file_listed(&harness, file_name).await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;
    (directory, harness, held, path, original, results)
}

async fn assert_cancel_finishes_without_touching_original(expectation: CancelExpectation<'_>) {
    let CancelExpectation {
        harness,
        results,
        job_id,
        relative_path,
        path,
        original,
        directory,
    } = expectation;
    let cancel_ack = harness.control(job_id, JobControl::Cancel).await.unwrap();
    assert!(
        cancel_ack.accepted,
        "cancel rejected: {}",
        cancel_ack.reason
    );
    assert_eq!(cancel_ack.status, CommandAckStatus::Canceling);

    wait_for_library_file_not_repairing(harness, relative_path).await;

    let (job, result) = next_job_result::<RepairRecordingResult>(results).await;
    assert_eq!(
        (job.status, job.reason.as_str()),
        (JobStatusStatus::Canceled, "")
    );
    assert_eq!(result.path, relative_path);
    assert_eq!(
        repair_job_status(harness, job_id).await,
        JobStatusStatus::Canceled
    );

    assert_eq!(fs::read(path).expect("read"), original);
    assert_eq!(
        recover_files(directory),
        Vec::<String>::new(),
        "cancel must remove the temporary file"
    );
}

async fn start_harness_with_interrupted_repair_job() -> (tempfile::TempDir, Harness<RecorderService>)
{
    let directory = tempdir().expect("tempdir");
    let settings_parent = tempdir().expect("settings");
    let path = directory.path().join("broken.mcap");
    write_truncated_mcap(&path);
    fs::write(directory.path().join("broken.recover"), b"temporary").expect("write recover");

    let mut persisted_jobs = Jobs::default();
    let goal = RepairRecordingGoal {
        path: "broken.mcap".into(),
    }
    .encode()
    .expect("encode");
    persisted_jobs
        .submit(
            JobId::from_u128(1),
            "RepairRecording",
            &goal,
            JobNature {
                lasting: true,
                ..JobNature::INSTANT
            },
        )
        .expect("submit");
    let envelope = PersistedRecorderState {
        version: NonZeroU32::MIN.get(),
        domain: RecorderDurableState,
        jobs: persisted_jobs,
    };
    let store = ServiceStateStore::open(
        RecorderService::NAME,
        Some(settings_parent.path().to_path_buf()),
        NonZeroU32::MIN,
    );
    fs::create_dir_all(store.path().parent().expect("state file parent directory")).expect("mkdir");
    fs::write(
        store.path(),
        serde_json::to_vec(&envelope).expect("serialize"),
    )
    .expect("write state");

    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let harness = Harness::<RecorderService>::start_on_with_context(
        Arc::clone(&backend),
        ServiceContext::with_settings_path(
            recorder_arguments(directory.path()),
            Some(settings_parent.path().to_path_buf()),
            backend,
        ),
    )
    .await
    .expect("harness");
    advance(Duration::from_millis(10)).await;
    drain_blocking_io().await;
    (directory, harness)
}

#[tokio::test(start_paused = true)]
async fn cancel_job_stops_a_held_repair_and_leaves_the_original_unchanged() {
    let (directory, harness, held, path, original, mut results) =
        held_truncated_repair_harness("cancel.mcap").await;

    let job_id = new_job_id();
    let repair_ack = harness
        .submit(
            "RepairRecording",
            job_id,
            &RepairRecordingGoal {
                path: "cancel.mcap".into(),
            },
        )
        .await
        .unwrap();
    assert!(
        repair_ack.accepted,
        "repair rejected: {}",
        repair_ack.reason
    );
    held.entered().notified().await;
    let repair_job_id = job_id.to_string();
    wait_for_library_state(harness.backend(), |library| {
        library.files.iter().any(|file| {
            file.path == "cancel.mcap"
                && file.repair_job_id == repair_job_id
                && file
                    .allowed_operations
                    .iter()
                    .any(|operation| operation == "CancelJob")
        })
    })
    .await;

    assert_cancel_finishes_without_touching_original(CancelExpectation {
        harness: &harness,
        results: &mut results,
        job_id,
        relative_path: "cancel.mcap",
        path: &path,
        original: &original,
        directory: directory.path(),
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn shutdown_stops_a_held_repair_before_it_finishes() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("held.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);
    let original = fs::read(&path).expect("read");

    let held = HeldRewrite::new();
    let harness = start_harness_with(directory.path(), |context| held.wrap(context)).await;
    wait_for_library_file_listed(&harness, "held.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let ack = harness
        .send(
            "RepairRecording",
            &RepairRecordingGoal {
                path: "held.mcap".into(),
            },
        )
        .await
        .unwrap();
    assert!(ack.accepted, "repair rejected: {}", ack.reason);
    held.entered().notified().await;

    timeout(Duration::from_secs(5), harness.shutdown())
        .await
        .expect("shutdown finishes");

    assert!(
        held.returned().load(Ordering::Relaxed),
        "shutdown must wait for the rewrite it cancelled"
    );
    assert_eq!(fs::read(&path).expect("read"), original);
    assert_eq!(
        recover_files(directory.path()),
        Vec::<String>::new(),
        "shutdown must remove the temporary file"
    );
}

#[tokio::test(start_paused = true)]
async fn restored_interrupted_repair_job_is_aborted_and_recover_discarded() {
    let (directory, harness) = start_harness_with_interrupted_repair_job().await;

    assert!(
        !directory.path().join("broken.recover").exists(),
        "startup must discard leftover recover files"
    );
    let job_list = harness.jobs().await.unwrap();
    let repair_job = job_list
        .jobs
        .iter()
        .find(|job| job.job_type == "RepairRecording")
        .expect("repair job in history");
    assert_eq!(
        (repair_job.status, repair_job.reason.as_str()),
        (JobStatusStatus::Aborted, "interrupted"),
        "interrupted repair must end aborted after restore"
    );
}
