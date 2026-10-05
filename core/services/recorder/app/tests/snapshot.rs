//! Recording snapshot through the Harness (layer L3, paused clock).

mod common;

use core::time::Duration;
use std::{fs, path::Path};

use bytes::Bytes;
use tempfile::tempdir;
use tokio::time::advance;

use blueos_comms::{Payload, Sample, Subscriber};
use blueos_idl::msg::{
    blueos_msgs::JobStatusStatus,
    blueos_recorder_msgs::{SnapshotRecordingGoal, SnapshotRecordingResult},
};
use blueos_recorder_app::RecorderService;
use blueos_recorder_mcap::{RECORDING_WRITE_CHUNK_SIZE, is_indexed};
use blueos_service::testing::Harness;

use common::harness::jobs::{next_job_result, subscribe_job_results};
use common::harness::recording::{
    active_recording_mcap_path, start_recording, stop_recording_and_finalize_mcap,
    stop_recording_on,
};
use common::harness::startup::start_harness;
use common::harness::state::{
    recording_state, wait_for_active_recording, wait_for_library_file_listed,
    wait_for_recording_bytes, wait_for_recording_bytes_on, wait_for_recording_idle,
};
use common::mcap_fixtures::write_truncated_mcap;

#[tokio::test(start_paused = true)]
async fn snapshot_active_recording_while_writer_runs() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    let mut results = subscribe_job_results(&harness, "SnapshotRecording").await;

    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    let payload_bytes = 128 * 1024;
    let sample_count = (RECORDING_WRITE_CHUNK_SIZE / payload_bytes as u64) as usize + 2;
    publish_chunk_fill_samples(&harness, sample_count, payload_bytes).await;
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
        .await
        .unwrap();
    assert!(ack.accepted, "snapshot rejected: {}", ack.reason);

    let snapshot_messages = assert_snapshot_succeeds(&harness, &directory, &mut results).await;

    let bytes_before_post_snapshot = recording_state(harness.backend())
        .await
        .session_bytes_written;
    publish_chunk_fill_samples(&harness, 4, payload_bytes).await;
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

async fn assert_snapshot_succeeds(
    harness: &Harness<RecorderService>,
    directory: &tempfile::TempDir,
    results: &mut Subscriber,
) -> usize {
    let (job, message) = next_job_result::<SnapshotRecordingResult>(results).await;
    assert_eq!(
        job.status,
        JobStatusStatus::Succeeded,
        "snapshot failed: {}",
        job.reason
    );
    assert!(!message.output_path.is_empty());

    wait_for_library_file_listed(harness, &message.output_path).await;
    let snapshot_path = directory.path().join(&message.output_path);
    assert!(is_indexed(&snapshot_path));
    let snapshot_messages = message_count(&snapshot_path);
    assert!(snapshot_messages > 0, "snapshot must contain messages");
    snapshot_messages
}

#[tokio::test(start_paused = true)]
async fn a_snapshot_of_the_active_recording_in_its_first_chunk_keeps_the_messages_on_disk() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    let mut results = subscribe_job_results(&harness, "SnapshotRecording").await;

    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    let payload_bytes = 16 * 1024;
    let sample_count = 16_u64;
    for seed in 0..sample_count {
        harness
            .backend()
            .publish(Sample::new(
                "snapshot/first_chunk",
                Payload::new(Bytes::from(incompressible_bytes(seed, payload_bytes))),
                "application/octet-stream",
            ))
            .await
            .expect("publish");
        advance(Duration::from_millis(20)).await;
    }
    let written = sample_count * payload_bytes as u64;
    assert!(written < RECORDING_WRITE_CHUNK_SIZE);
    wait_for_recording_bytes(&harness, written).await;

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
                path: recording_path,
            },
        )
        .await
        .unwrap();
    assert!(ack.accepted, "snapshot rejected: {}", ack.reason);

    let (job, message) = next_job_result::<SnapshotRecordingResult>(&mut results).await;
    assert_eq!(
        job.status,
        JobStatusStatus::Succeeded,
        "snapshot failed: {}",
        job.reason
    );
    let snapshot_path = directory.path().join(&message.output_path);
    assert!(is_indexed(&snapshot_path));
    assert!(
        message_count(&snapshot_path) > 0,
        "the snapshot must keep the messages of the open chunk that reached the disk"
    );
}

#[tokio::test(start_paused = true)]
async fn snapshot_rejects_a_recording_that_is_not_being_written() {
    let directory = tempdir().expect("tempdir");
    write_truncated_mcap(&directory.path().join("partial.mcap"));

    let harness = start_harness(directory.path()).await;
    stop_recording_on(harness.backend()).await;
    wait_for_recording_idle(harness.backend()).await;
    wait_for_library_file_listed(&harness, "partial.mcap").await;

    let ack = harness
        .send(
            "SnapshotRecording",
            &SnapshotRecordingGoal {
                path: "partial.mcap".into(),
            },
        )
        .await
        .unwrap();
    assert!(!ack.accepted, "a finished recording downloads directly");
    assert_eq!(
        ack.reason,
        "Only the recording being written needs a snapshot. Download it directly."
    );
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
        .await
        .unwrap();
    assert!(!ack.accepted, "missing file must be rejected");
}

fn message_count(path: &Path) -> usize {
    let bytes = fs::read(path).expect("read mcap");
    mcap::MessageStream::new(&bytes).expect("stream").count()
}

/// Bytes LZ4 cannot shrink, so the compressed chunk reaches the disk as fast as samples arrive. Each `seed` gives
/// different bytes, so samples do not repeat each other either.
async fn publish_chunk_fill_samples(
    harness: &Harness<RecorderService>,
    sample_count: usize,
    payload_bytes: usize,
) {
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
}

fn incompressible_bytes(seed: u64, length: usize) -> Vec<u8> {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64 ^ seed;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state.to_le_bytes()[0]
        })
        .collect()
}
