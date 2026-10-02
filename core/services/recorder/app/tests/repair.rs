//! Native repair through the Harness (layer L3, paused clock).

use core::time::Duration;
use std::{fs, path::Path, process::Command, sync::Arc};

use mcap::{Writer, write::WriteOptions};
use tempfile::tempdir;
use tokio::time::{advance, timeout};

use blueos_api::event_key;
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_idl::{
    Message,
    msg::blueos_recorder_msgs::{
        CancelRepairCommand, RecordingFileState, RecordingLibrary, RecordingOperation,
        RecordingOperationOperation, RepairRecordingCommand,
    },
};
use blueos_recorder_app::{RecorderArguments, RecorderService};
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_recorder_mcap::is_indexed;
use blueos_service::{
    Service,
    testing::{Harness, WALL_CLOCK_AT_START},
};

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
            mcap_writer_queue_capacity: None,
        },
    )
    .await
    .expect("harness");

    wait_for_library_file(&harness, "broken.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_rescan_io(&harness).await;

    let ack = harness
        .send(
            "RepairRecording",
            &RepairRecordingCommand {
                path: "broken.mcap".into(),
            },
        )
        .await;
    assert!(ack.accepted, "repair rejected: {}", ack.reason);

    for _ in 0..400 {
        advance(Duration::from_millis(50)).await;
        let library = harness.state::<RecordingLibrary>("library").await;
        if library
            .files
            .iter()
            .any(|file| file.state == RecordingFileState::Ready)
        {
            break;
        }
    }

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
        mcap_writer_queue_capacity: None,
    })
    .await
    .expect("harness");

    wait_for_library_file(&harness, "cancel.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_rescan_io(&harness).await;

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

    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
    }

    assert_eq!(fs::read(&path).expect("read"), original);
    assert!(
        !directory.path().join("cancel.recover").exists(),
        "cancel must remove the temporary file"
    );
}

#[tokio::test(start_paused = true)]
async fn leftover_recover_file_is_removed_at_startup() {
    let directory = tempdir().expect("tempdir");
    fs::write(directory.path().join("stale.recover"), b"leftover").expect("write");

    let _harness = Harness::<RecorderService>::start(RecorderArguments {
        recorder_path: directory.path().to_path_buf(),
        mcap_writer_queue_capacity: None,
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

async fn wait_for_library_file(harness: &Harness<RecorderService>, name: &str) {
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
        let library = harness.state::<RecordingLibrary>("library").await;
        if library.files.iter().any(|file| file.path == name) {
            return;
        }
    }
    panic!("library never listed {name}");
}

async fn drain_rescan_io(harness: &Harness<RecorderService>) {
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
        let _library: RecordingLibrary = harness.state("library").await;
    }
}
