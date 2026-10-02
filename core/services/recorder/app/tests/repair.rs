//! Native repair through the Harness (layer L3, paused clock).

mod common;

use core::{num::NonZeroU32, time::Duration};
use std::{fs, path::Path, process::Command, sync::Arc};

use mcap::{Writer, write::WriteOptions};
use serde::Serialize;
use tempfile::tempdir;
use tokio::time::{advance, timeout};

use blueos_api::event_key;
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_idl::{
    Message,
    msg::blueos_msgs::JobStatusStatus,
    msg::blueos_recorder_msgs::{
        CancelRepairCommand, RecordingOperation, RecordingOperationOperation,
        RepairRecordingCommand,
    },
};
use blueos_jobs::{JobGraph, Jobs};
use blueos_recorder_app::{RecorderArguments, RecorderService};
use blueos_recorder_domain::{durable::RecorderDurableState, job::RecorderJobStep};
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_recorder_mcap::is_indexed;
use blueos_recorder_paths::RecordingRelativePath;
use blueos_service::{
    Service, ServiceContext,
    testing::{Harness, WALL_CLOCK_AT_START},
};
use blueos_settings::ServiceStateStore;

use common::{
    drain_blocking_io, wait_for_library_file_listed, wait_for_library_file_not_repairing,
    wait_for_library_file_ready,
};

#[derive(Serialize)]
struct PersistedRecorderState {
    #[serde(rename = "VERSION")]
    version: u32,
    domain: RecorderDurableState,
    jobs: Jobs<RecorderJobStep>,
}

#[tokio::test(start_paused = true)]
async fn repair_rewrites_truncated_recording_and_publishes_operation_event() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("broken.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);

    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let operation_key = event_key(RecorderService::NAME, "operation");
    let mut operations = backend.subscribe(&operation_key).await.expect("subscribe");

    let harness = Harness::start_on(
        Arc::clone(&backend),
        RecorderArguments {
            recorder_path: directory.path().to_path_buf(),
        },
    )
    .await
    .expect("harness");

    wait_for_library_file_listed(&harness, "broken.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let ack = harness
        .send(
            "RepairRecording",
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
}

#[tokio::test(start_paused = true)]
async fn cancel_repair_leaves_original_bytes_unchanged() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("cancel.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);
    let original = fs::read(&path).expect("read");

    let harness = Harness::<RecorderService>::start(RecorderArguments {
        recorder_path: directory.path().to_path_buf(),
    })
    .await
    .expect("harness");

    wait_for_library_file_listed(&harness, "cancel.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    harness
        .send(
            "RepairRecording",
            &RepairRecordingCommand {
                path: "cancel.mcap".into(),
            },
        )
        .await;

    advance(Duration::from_millis(100)).await;

    harness
        .send(
            "CancelRepair",
            &CancelRepairCommand {
                path: "cancel.mcap".into(),
            },
        )
        .await;

    wait_for_library_file_not_repairing(&harness, "cancel.mcap").await;

    assert_eq!(fs::read(&path).expect("read"), original);
    assert!(
        !directory.path().join("cancel.recover").exists(),
        "cancel must remove the temporary file"
    );
}

#[tokio::test(start_paused = true)]
async fn restored_interrupted_repair_job_is_failed_and_recover_discarded() {
    let directory = tempdir().expect("tempdir");
    let settings_parent = tempdir().expect("settings");
    let path = directory.path().join("broken.mcap");
    write_truncated_mcap(&path);
    fs::write(directory.path().join("broken.recover"), b"temporary").expect("write recover");

    let mut persisted_jobs = Jobs::default();
    persisted_jobs.start(JobGraph::Leaf(RecorderJobStep::RepairRecording {
        path: RecordingRelativePath::parse("broken.mcap").expect("path"),
    }));
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
    let context = ServiceContext::with_settings_path(
        RecorderArguments {
            recorder_path: directory.path().to_path_buf(),
        },
        Some(settings_parent.path().to_path_buf()),
        Arc::clone(&backend),
    );
    let harness = Harness::<RecorderService>::start_on_with_context(backend, context)
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
        .find(|job| job.name.starts_with("repair "))
        .expect("repair job in history");
    assert_eq!(
        repair_job.status,
        JobStatusStatus::Failed,
        "interrupted repair must finish as failed after restore"
    );
}

#[tokio::test(start_paused = true)]
async fn leftover_recover_file_is_removed_at_startup() {
    let directory = tempdir().expect("tempdir");
    fs::write(directory.path().join("stale.recover"), b"leftover").expect("write");

    let _harness = Harness::<RecorderService>::start(RecorderArguments {
        recorder_path: directory.path().to_path_buf(),
    })
    .await
    .expect("harness");

    assert!(
        !directory.path().join("stale.recover").exists(),
        "startup must discard leftover recover files"
    );
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
