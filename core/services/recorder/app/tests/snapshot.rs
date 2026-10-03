//! Recording snapshot through the Harness (layer L3, paused clock).

mod common;

use core::time::Duration;
use std::{fs, path::Path};

use bytes::Bytes;
use mcap::{Writer, write::WriteOptions};
use tempfile::tempdir;
use tokio::time::advance;

use blueos_comms::{Payload, Sample};
use blueos_idl::msg::{
    blueos_msgs::JobStatusStatus,
    blueos_recorder_msgs::{SnapshotRecordingGoal, SnapshotRecordingResult},
};
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_recorder_mcap::{RECORDING_WRITE_CHUNK_SIZE, is_indexed};

use common::{
    active_recording_mcap_path, drain_blocking_io, next_job_result, recording_state, start_harness,
    start_recording, stop_recording_and_finalize_mcap, stop_recording_on, subscribe_job_results,
    wait_for_active_recording, wait_for_library_file_listed, wait_for_recording_bytes,
    wait_for_recording_bytes_on, wait_for_recording_idle,
};

#[tokio::test(start_paused = true)]
async fn snapshot_active_recording_while_writer_runs() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    let mut results = subscribe_job_results(&harness, "SnapshotRecording").await;

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
    let recording_path = recording_file
        .file_name()
        .expect("file name")
        .to_string_lossy()
        .into_owned();
    wait_for_library_file_listed(&harness, &recording_path).await;

    let ack = harness
        .send(
            "SnapshotRecording",
            &SnapshotRecordingGoal {
                path: recording_path.clone(),
            },
        )
        .await;
    assert!(ack.accepted, "snapshot rejected: {}", ack.reason);

    let (job, message) = next_job_result::<SnapshotRecordingResult>(&mut results).await;
    assert_eq!(
        job.status,
        JobStatusStatus::Succeeded,
        "snapshot failed: {}",
        job.reason
    );
    assert!(!message.output_path.is_empty());

    wait_for_library_file_listed(&harness, &message.output_path).await;
    let snapshot_path = directory.path().join(&message.output_path);
    assert!(is_indexed(&snapshot_path));
    let snapshot_messages = message_count(&snapshot_path);
    assert!(snapshot_messages > 0, "snapshot must contain messages");

    let bytes_before_post_snapshot = recording_state(harness.backend())
        .await
        .session_bytes_written;
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
    wait_for_recording_bytes_on(
        harness.backend(),
        bytes_before_post_snapshot + 4 * payload_bytes as u64,
    )
    .await;

    stop_recording_and_finalize_mcap(harness.backend(), &recording_file).await;
    assert!(is_indexed(&recording_file));
    let final_messages = message_count(&recording_file);
    assert!(
        final_messages > snapshot_messages,
        "live recording must gain messages after the snapshot"
    );
}

#[tokio::test(start_paused = true)]
async fn snapshot_job_result_names_the_indexed_snapshot() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("partial.mcap");
    write_truncated_mcap(&path);

    let harness = start_harness(directory.path()).await;
    let mut results = subscribe_job_results(&harness, "SnapshotRecording").await;

    stop_recording_on(harness.backend()).await;
    wait_for_recording_idle(harness.backend()).await;

    wait_for_library_file_listed(&harness, "partial.mcap").await;
    advance(RESCAN_INTERVAL).await;
    drain_blocking_io().await;

    let ack = harness
        .send(
            "SnapshotRecording",
            &SnapshotRecordingGoal {
                path: "partial.mcap".into(),
            },
        )
        .await;
    assert!(ack.accepted, "snapshot rejected: {}", ack.reason);

    let (job, message) = next_job_result::<SnapshotRecordingResult>(&mut results).await;
    assert_eq!(
        job.status,
        JobStatusStatus::Succeeded,
        "snapshot failed: {}",
        job.reason
    );
    assert!(!message.output_path.is_empty());
    assert_eq!(message.path, "partial.mcap");

    wait_for_library_file_listed(&harness, &message.output_path).await;
    let snapshot_path = directory.path().join(&message.output_path);
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
    wait_for_recording_idle(harness.backend()).await;

    let ack = harness
        .send(
            "SnapshotRecording",
            &SnapshotRecordingGoal {
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
