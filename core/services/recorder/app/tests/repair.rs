//! Native repair through the Harness (layer L3, paused clock).

mod common;

use core::time::Duration;
use std::fs;

use tempfile::tempdir;
use tokio::time::advance;

use blueos_api::job_feedback_key;
use blueos_idl::{
    msg::blueos_msgs::JobStatusStatus,
    msg::blueos_recorder_msgs::{RepairRecordingGoal, RepairRecordingResult},
};
use blueos_recorder_app::RecorderService;
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_recorder_mcap::is_indexed;
use blueos_service::Service;
use blueos_service::new_job_id;

use common::harness::io::{drain_blocking_io, set_modified_seconds_ago};
use common::harness::jobs::{
    next_job_result, read_offsets, repair_job_status, subscribe_job_results, wait_for_read_offset,
};
use common::harness::startup::{start_harness, start_harness_with};
use common::harness::state::{
    wait_for_library_file_listed, wait_for_library_file_ready, wait_for_library_state,
};
use common::mcap_fixtures::{write_mcap_killed_before_its_first_message, write_truncated_mcap};
use common::repair_rewrite::SteppedRewrite;

#[tokio::test(start_paused = true)]
async fn repair_rewrites_truncated_recording_and_publishes_its_job_result() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("broken.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);

    let harness = start_harness(directory.path()).await;
    let mut results = subscribe_job_results(&harness, "RepairRecording").await;

    wait_for_library_file_listed(&harness, "broken.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let job_id = new_job_id();
    let ack = harness
        .submit(
            "RepairRecording",
            job_id,
            &RepairRecordingGoal {
                path: "broken.mcap".into(),
            },
        )
        .await
        .unwrap();
    assert!(ack.accepted, "repair rejected: {}", ack.reason);

    wait_for_library_file_ready(&harness, "broken.mcap").await;

    assert!(is_indexed(&path), "repaired file must be indexed on disk");

    let (job, result) = next_job_result::<RepairRecordingResult>(&mut results).await;
    assert_eq!(
        (job.job_id, job.status, job.reason.as_str()),
        (job_id.to_string(), JobStatusStatus::Succeeded, "")
    );
    assert_eq!(result.path, "broken.mcap");
    assert_eq!(
        repair_job_status(&harness, job_id).await,
        JobStatusStatus::Succeeded
    );
}

#[tokio::test(start_paused = true)]
async fn a_recording_killed_before_its_first_message_is_repaired_empty_and_no_longer_offered_for_repair()
 {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("killed.mcap");
    write_mcap_killed_before_its_first_message(&path);
    set_modified_seconds_ago(&path, 20);

    let harness = start_harness(directory.path()).await;
    let mut results = subscribe_job_results(&harness, "RepairRecording").await;
    wait_for_library_file_listed(&harness, "killed.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let ack = harness
        .send(
            "RepairRecording",
            &RepairRecordingGoal {
                path: "killed.mcap".into(),
            },
        )
        .await
        .unwrap();
    assert!(ack.accepted, "repair rejected: {}", ack.reason);

    let (job, _result) = next_job_result::<RepairRecordingResult>(&mut results).await;
    assert_eq!(
        (job.status, job.reason.as_str()),
        (JobStatusStatus::Succeeded, "")
    );
    wait_for_library_file_ready(&harness, "killed.mcap").await;
    let bytes = fs::read(&path).expect("read the repaired recording");
    assert_eq!(mcap::MessageStream::new(&bytes).expect("stream").count(), 0);
    wait_for_library_state(harness.backend(), |library| {
        library.files.iter().any(|file| {
            file.path == "killed.mcap"
                && !file
                    .allowed_operations
                    .iter()
                    .any(|operation| operation == "RepairRecording")
        })
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn a_file_that_is_not_an_mcap_aborts_its_repair_and_is_not_offered_repair_again() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("notes.mcap");
    fs::write(&path, b"not an MCAP recording").expect("write");
    set_modified_seconds_ago(&path, 20);

    let harness = start_harness(directory.path()).await;
    let mut results = subscribe_job_results(&harness, "RepairRecording").await;
    wait_for_library_file_listed(&harness, "notes.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let ack = harness
        .send(
            "RepairRecording",
            &RepairRecordingGoal {
                path: "notes.mcap".into(),
            },
        )
        .await
        .unwrap();
    assert!(ack.accepted, "repair rejected: {}", ack.reason);

    let (job, _result) = next_job_result::<RepairRecordingResult>(&mut results).await;
    assert_eq!(job.status, JobStatusStatus::Aborted);
    advance(RESCAN_INTERVAL).await;
    drain_blocking_io().await;
    wait_for_library_state(harness.backend(), |library| {
        library.files.iter().any(|file| {
            file.path == "notes.mcap"
                && file.repair_error == "This is not an MCAP file."
                && !file
                    .allowed_operations
                    .iter()
                    .any(|operation| operation == "RepairRecording")
        })
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn repair_feedback_reports_a_growing_read_offset_also_to_a_client_that_opens_it_mid_repair() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("progress.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);
    let total_bytes = fs::metadata(&path).expect("metadata").len();

    let stepped = SteppedRewrite::new();
    let harness = start_harness_with(directory.path(), |context| stepped.wrap(context)).await;
    wait_for_library_file_listed(&harness, "progress.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let mut feedback = harness
        .backend()
        .subscribe(&job_feedback_key(RecorderService::NAME, "RepairRecording"))
        .await
        .expect("subscribe to the repair Feedback");
    let mut results = subscribe_job_results(&harness, "RepairRecording").await;
    let job_id = new_job_id();
    let ack = harness
        .submit(
            "RepairRecording",
            job_id,
            &RepairRecordingGoal {
                path: "progress.mcap".into(),
            },
        )
        .await
        .unwrap();
    assert!(ack.accepted, "repair rejected: {}", ack.reason);

    let first = total_bytes / 3;
    stepped.report(first);
    wait_for_read_offset(&mut feedback, job_id, first).await;
    let late = harness.job_feedback("RepairRecording").await.unwrap();
    assert_eq!(
        read_offsets(&late, job_id),
        Some((first, total_bytes)),
        "a client that opens the Feedback mid-repair sees the latest read offset"
    );

    let second = 2 * total_bytes / 3;
    stepped.report(second);
    wait_for_read_offset(&mut feedback, job_id, second).await;

    stepped.finish();
    let (job, _result) = next_job_result::<RepairRecordingResult>(&mut results).await;
    assert_eq!(job.status, JobStatusStatus::Succeeded);
    assert_eq!(
        read_offsets(
            &harness.job_feedback("RepairRecording").await.unwrap(),
            job_id
        ),
        None,
        "a Job leaves the Feedback when it ends"
    );
}
