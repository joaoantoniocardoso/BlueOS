//! Recording snapshot through the Harness (layer L3, paused clock).

mod common;

use core::time::Duration;
use std::{fs, path::Path, sync::Arc};

use bytes::Bytes;
use mcap::{Writer, write::WriteOptions};
use tempfile::tempdir;
use tokio::time::{advance, timeout};

use blueos_api::event_key;
use blueos_comms::{CommsBackend, Payload, Sample, channel::ChannelBackend};
use blueos_idl::{
    Message,
    msg::blueos_recorder_msgs::{
        RecordingLibrary, RecordingOperation, RecordingOperationOperation, SnapshotRecordingCommand,
    },
};
use blueos_recorder_app::RecorderService;
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_recorder_mcap::{RECORDING_WRITE_CHUNK_SIZE, is_indexed};
use blueos_service::{Service, testing::Harness};

use common::{
    active_recording_mcap_path, recorder_arguments, start_harness, start_recording,
    stop_recording_and_finalize_mcap, stop_recording_on, wait_for_active_recording,
    wait_for_recording_bytes,
};

#[tokio::test(start_paused = true)]
async fn snapshot_active_recording_while_writer_runs() {
    let directory = tempdir().expect("tempdir");
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let operation_key = event_key(RecorderService::NAME, "operation");
    let mut operations = backend.subscribe(&operation_key).await.expect("subscribe");

    let harness = Harness::start_on(
        Arc::clone(&backend),
        recorder_arguments(directory.path(), None),
    )
    .await
    .expect("harness");

    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    let payload_bytes = 128 * 1024;
    let sample_count = (RECORDING_WRITE_CHUNK_SIZE / payload_bytes as u64) as usize + 2;
    for _ in 0..sample_count {
        harness
            .backend()
            .publish(Sample::new(
                "snapshot/chunk_fill",
                Payload::new(Bytes::from(vec![0_u8; payload_bytes])),
                "application/octet-stream",
            ))
            .await
            .expect("publish");
        advance(Duration::from_millis(20)).await;
    }
    wait_for_recording_bytes(&harness, RECORDING_WRITE_CHUNK_SIZE).await;

    let recording_file = active_recording_mcap_path(&harness, directory.path()).await;
    wait_for_recording_file_bytes(&recording_file, 4096).await;
    let recording_path = recording_file
        .file_name()
        .expect("file name")
        .to_string_lossy()
        .into_owned();
    wait_for_library_file(&harness, &recording_path).await;

    let ack = harness
        .send(
            "SnapshotRecording",
            &SnapshotRecordingCommand {
                path: recording_path.clone(),
            },
        )
        .await;
    assert!(ack.accepted, "snapshot rejected: {}", ack.reason);

    let operation = timeout(Duration::from_secs(5), operations.recv())
        .await
        .expect("operation event")
        .expect("payload");
    let message =
        RecordingOperation::decode(operation.payload().to_bytes().as_ref()).expect("decode");
    assert_eq!(message.operation, RecordingOperationOperation::Snapshot);
    assert!(message.succeeded, "snapshot failed: {}", message.error);
    assert!(!message.output_path.is_empty());

    let snapshot_path = directory.path().join(&message.output_path);
    wait_for_recording_file_bytes(&snapshot_path, 1).await;
    assert!(is_indexed(&snapshot_path));
    let snapshot_messages = message_count(&snapshot_path);
    assert!(snapshot_messages > 0, "snapshot must contain messages");

    for _ in 0..4 {
        harness
            .backend()
            .publish(Sample::new(
                "snapshot/chunk_fill",
                Payload::new(Bytes::from(vec![0_u8; payload_bytes])),
                "application/octet-stream",
            ))
            .await
            .expect("publish");
        advance(Duration::from_millis(20)).await;
    }

    stop_recording_and_finalize_mcap(harness.backend(), &recording_file).await;
    assert!(is_indexed(&recording_file));
    let final_messages = message_count(&recording_file);
    assert!(
        final_messages > snapshot_messages,
        "live recording must gain messages after the snapshot"
    );
}

#[tokio::test(start_paused = true)]
async fn snapshot_rewrite_publishes_indexed_output_path() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("partial.mcap");
    write_truncated_mcap(&path);

    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let operation_key = event_key(RecorderService::NAME, "operation");
    let mut operations = backend.subscribe(&operation_key).await.expect("subscribe");

    let harness = Harness::start_on(
        Arc::clone(&backend),
        recorder_arguments(directory.path(), None),
    )
    .await
    .expect("harness");

    stop_recording_on(harness.backend()).await;
    for _ in 0..100 {
        advance(Duration::from_millis(50)).await;
    }

    wait_for_library_file(&harness, "partial.mcap").await;
    advance(RESCAN_INTERVAL).await;
    drain_rescan(&harness).await;

    let ack = harness
        .send(
            "SnapshotRecording",
            &SnapshotRecordingCommand {
                path: "partial.mcap".into(),
            },
        )
        .await;
    assert!(ack.accepted, "snapshot rejected: {}", ack.reason);

    let operation = timeout(Duration::from_secs(5), operations.recv())
        .await
        .expect("operation event")
        .expect("payload");
    let message =
        RecordingOperation::decode(operation.payload().to_bytes().as_ref()).expect("decode");
    assert_eq!(message.operation, RecordingOperationOperation::Snapshot);
    assert!(message.succeeded, "snapshot failed: {}", message.error);
    assert!(!message.output_path.is_empty());
    assert_eq!(message.path, "partial.mcap");

    let snapshot_path = directory.path().join(&message.output_path);
    wait_for_recording_file_bytes(&snapshot_path, 1).await;
    assert!(snapshot_path.exists(), "snapshot file must exist on disk");
    assert!(is_indexed(&snapshot_path));
    let bytes = fs::read(&snapshot_path).expect("read snapshot");
    let summary = mcap::Summary::read(&bytes)
        .expect("summary read")
        .expect("summary");
    assert!(!summary.chunk_indexes.is_empty());
    let messages = mcap::MessageStream::new(&bytes).expect("stream").count();
    assert!(messages > 0);
}

#[tokio::test(start_paused = true)]
async fn snapshot_rejects_missing_file() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    stop_recording_on(harness.backend()).await;
    for _ in 0..100 {
        advance(Duration::from_millis(50)).await;
    }

    let ack = harness
        .send(
            "SnapshotRecording",
            &SnapshotRecordingCommand {
                path: "missing.mcap".into(),
            },
        )
        .await;
    assert!(!ack.accepted, "missing file must be rejected");
}

fn message_count(path: &Path) -> usize {
    let bytes = fs::read(path).expect("read mcap");
    mcap::MessageStream::new(&bytes).expect("stream").count()
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

async fn wait_for_recording_file_bytes(path: &Path, minimum: u64) {
    let path = path.to_path_buf();
    for _ in 0..600 {
        advance(Duration::from_millis(50)).await;
        let size = tokio::task::spawn_blocking({
            let path = path.clone();
            move || {
                fs::metadata(&path)
                    .map(|metadata| metadata.len())
                    .unwrap_or(0)
            }
        })
        .await
        .unwrap_or(0);
        if size >= minimum {
            return;
        }
    }
    panic!("file never reached {minimum} bytes on disk");
}

async fn wait_for_library_file(harness: &Harness<RecorderService>, path: &str) {
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
        let library = harness.state::<RecordingLibrary>("library").await;
        if library.files.iter().any(|file| file.path == path) {
            return;
        }
    }
    panic!("library never listed {path}");
}

async fn drain_rescan(harness: &Harness<RecorderService>) {
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
        let _library: RecordingLibrary = harness.state("library").await;
    }
}
