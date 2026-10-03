//! Native repair through the Harness (layer L3, paused clock).

mod common;

use core::{
    num::NonZeroU32,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::{fs, path::Path, process::Command, sync::Arc};

use mcap::{Writer, write::WriteOptions};
use serde::Serialize;
use tempfile::tempdir;
use tokio::{
    sync::Notify,
    time::{advance, timeout},
};

use blueos_api::event_key;
use blueos_idl::{
    Message,
    msg::blueos_msgs::{CommandAckStatus, JobStatusStatus},
    msg::blueos_recorder_msgs::{
        RecordingOperation, RecordingOperationOperation, RepairRecordingCommand,
    },
};
use blueos_jobs::{JobControl, JobId, JobNature, Jobs};
use blueos_recorder_app::{RecorderContext, RecorderService};
use blueos_recorder_domain::durable::RecorderDurableState;
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_recorder_mcap::is_indexed;
use blueos_service::{
    Service, new_job_id,
    testing::{Harness, WALL_CLOCK_AT_START},
};
use blueos_settings::ServiceStateStore;

use common::{
    drain_blocking_io, recorder_arguments, start_harness, start_harness_with,
    wait_for_library_file_listed, wait_for_library_file_not_repairing, wait_for_library_file_ready,
    wait_for_library_state,
};

/// Holds the real rewrite Port mid-flight until the rewrite is cancelled or the test ends.
struct HeldRewrite {
    entered: Arc<Notify>,
    release: Arc<AtomicBool>,
    returned: Arc<AtomicBool>,
}

#[derive(Serialize)]
struct PersistedRecorderState {
    #[serde(rename = "VERSION")]
    version: u32,
    domain: RecorderDurableState,
    jobs: Jobs,
}

impl Drop for HeldRewrite {
    fn drop(&mut self) {
        self.release.store(true, Ordering::Relaxed);
    }
}

impl HeldRewrite {
    fn new() -> Self {
        Self {
            entered: Arc::new(Notify::new()),
            release: Arc::new(AtomicBool::new(false)),
            returned: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Wraps the rewrite Port in `context`: it waits for a cancel or the release, then runs the real rewrite.
    fn wrap(&self, context: &mut RecorderContext) {
        let rewriter = Arc::clone(&context.rewriter);
        let entered = Arc::clone(&self.entered);
        let release = Arc::clone(&self.release);
        let returned = Arc::clone(&self.returned);
        context.rewriter = Arc::new(move |source, output, progress, cancel| {
            entered.notify_one();
            while !release.load(Ordering::Relaxed) && !cancel.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
            let rewritten = rewriter(source, output, progress, cancel);
            returned.store(true, Ordering::Relaxed);
            rewritten
        });
    }
}

#[tokio::test(start_paused = true)]
async fn repair_rewrites_truncated_recording_and_publishes_operation_event() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("broken.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);

    let harness = start_harness(directory.path()).await;
    let operation_key = event_key(RecorderService::NAME, "operation");
    let mut operations = harness
        .backend()
        .subscribe(&operation_key)
        .await
        .expect("subscribe");

    wait_for_library_file_listed(&harness, "broken.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let job_id = new_job_id();
    let ack = harness
        .submit(
            "RepairRecording",
            job_id,
            &RepairRecordingCommand {
                path: "broken.mcap".into(),
            },
        )
        .await;
    assert!(ack.accepted, "repair rejected: {}", ack.reason);

    wait_for_library_file_ready(&harness, "broken.mcap").await;

    assert!(is_indexed(&path), "repaired file must be indexed on disk");

    let operation = timeout(Duration::from_secs(5), operations.recv())
        .await
        .expect("operation event")
        .expect("payload");
    let message =
        RecordingOperation::decode(operation.payload().to_bytes().as_ref()).expect("decode");
    assert_eq!(message.operation, RecordingOperationOperation::Repair);
    assert!(message.succeeded);
    assert!(!message.cancelled);
    assert!(message.error.is_empty());
    assert_eq!(
        repair_job_status(&harness, job_id).await,
        JobStatusStatus::Succeeded
    );
}

#[tokio::test(start_paused = true)]
async fn cancel_job_stops_a_held_repair_and_leaves_the_original_unchanged() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("cancel.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);
    let original = fs::read(&path).expect("read");

    let held = HeldRewrite::new();
    let harness = start_harness_with(directory.path(), |context| held.wrap(context)).await;

    let operation_key = event_key(RecorderService::NAME, "operation");
    let mut operations = harness
        .backend()
        .subscribe(&operation_key)
        .await
        .expect("subscribe");

    wait_for_library_file_listed(&harness, "cancel.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let job_id = new_job_id();
    let repair_ack = harness
        .submit(
            "RepairRecording",
            job_id,
            &RepairRecordingCommand {
                path: "cancel.mcap".into(),
            },
        )
        .await;
    assert!(
        repair_ack.accepted,
        "repair rejected: {}",
        repair_ack.reason
    );
    held.entered.notified().await;
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

    let cancel_ack = harness.control(job_id, JobControl::Cancel).await;
    assert!(
        cancel_ack.accepted,
        "cancel rejected: {}",
        cancel_ack.reason
    );
    assert_eq!(cancel_ack.status, CommandAckStatus::Canceling);

    wait_for_library_file_not_repairing(&harness, "cancel.mcap").await;

    let operation = timeout(Duration::from_secs(5), operations.recv())
        .await
        .expect("operation event")
        .expect("payload");
    let message =
        RecordingOperation::decode(operation.payload().to_bytes().as_ref()).expect("decode");
    assert_eq!(message.operation, RecordingOperationOperation::Repair);
    assert!(!message.succeeded);
    assert!(message.cancelled);
    assert!(message.error.is_empty());
    assert_eq!(
        repair_job_status(&harness, job_id).await,
        JobStatusStatus::Canceled
    );

    assert_eq!(fs::read(&path).expect("read"), original);
    assert!(
        !directory.path().join("cancel.recover").exists(),
        "cancel must remove the temporary file"
    );
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
            &RepairRecordingCommand {
                path: "held.mcap".into(),
            },
        )
        .await;
    assert!(ack.accepted, "repair rejected: {}", ack.reason);
    held.entered.notified().await;

    timeout(Duration::from_secs(5), harness.shutdown())
        .await
        .expect("shutdown finishes");

    assert!(
        held.returned.load(Ordering::Relaxed),
        "shutdown must wait for the rewrite it cancelled"
    );
    assert_eq!(fs::read(&path).expect("read"), original);
    assert!(
        !directory.path().join("held.recover").exists(),
        "shutdown must remove the temporary file"
    );
}

#[tokio::test(start_paused = true)]
async fn restored_interrupted_repair_job_is_aborted_and_recover_discarded() {
    let directory = tempdir().expect("tempdir");
    let settings_parent = tempdir().expect("settings");
    let path = directory.path().join("broken.mcap");
    write_truncated_mcap(&path);
    fs::write(directory.path().join("broken.recover"), b"temporary").expect("write recover");

    let mut persisted_jobs = Jobs::default();
    let goal = RepairRecordingCommand {
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

    let harness = Harness::<RecorderService>::start_with_settings_path(
        recorder_arguments(directory.path()),
        Some(settings_parent.path().to_path_buf()),
        |_context| {},
    )
    .await
    .expect("harness");
    advance(Duration::from_millis(10)).await;
    drain_blocking_io().await;

    assert!(
        !directory.path().join("broken.recover").exists(),
        "startup must discard leftover recover files"
    );
    let job_list = harness.jobs().await;
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

#[tokio::test(start_paused = true)]
async fn leftover_recover_file_is_removed_at_startup() {
    let directory = tempdir().expect("tempdir");
    fs::write(directory.path().join("stale.recover"), b"leftover").expect("write");

    let _harness = start_harness(directory.path()).await;

    assert!(
        !directory.path().join("stale.recover").exists(),
        "startup must discard leftover recover files"
    );
}

async fn repair_job_status(harness: &Harness<RecorderService>, job_id: JobId) -> JobStatusStatus {
    let job_id = job_id.to_string();
    harness
        .jobs()
        .await
        .jobs
        .into_iter()
        .find(|job| job.job_id == job_id)
        .expect("the repair Job is in the jobs State")
        .status
}

fn set_modified_seconds_ago(path: &Path, seconds_ago: u64) {
    let stamp = WALL_CLOCK_AT_START.as_secs().saturating_sub(seconds_ago);
    let status = Command::new("touch")
        .args([
            "-d",
            &format!("@{stamp}"),
            path.to_str().expect("utf8 path"),
        ])
        .status()
        .expect("touch");
    assert!(
        status.success(),
        "touch must set an old mtime for the 10 s repair rule"
    );
}

fn write_truncated_mcap(path: &Path) {
    let file = fs::File::create(path).expect("create");
    let mut writer = Writer::with_options(file, WriteOptions::new()).expect("writer");
    let schema_id = writer
        .add_schema("test", "jsonschema", b"{}")
        .expect("schema");
    let channel_id = writer
        .add_channel(schema_id, "topic", "json", &Default::default())
        .expect("channel");
    for index in 0..8 {
        let header = mcap::records::MessageHeader {
            channel_id,
            sequence: index as u32,
            log_time: index as u64,
            publish_time: index as u64,
        };
        writer
            .write_to_known_channel(&header, b"payload")
            .expect("write");
    }
    writer.finish().expect("finish");
    let bytes = fs::read(path).expect("read");
    fs::write(path, &bytes[..bytes.len() / 2]).expect("truncate");
}
