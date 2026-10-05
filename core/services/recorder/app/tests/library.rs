//! Recorder library State and delete (layer L3, paused clock).

mod common;

use core::time::Duration;
use std::{fs, path::PathBuf};

use blueos_comms::Subscriber;
use tempfile::{TempDir, tempdir};
use tokio::time::{advance, timeout};

use blueos_api::state_key;
use blueos_idl::msg::{
    blueos_msgs::{CommandAckStatus, JobStatusStatus},
    blueos_recorder_msgs::{
        DeleteRecordingGoal, DeleteRecordingResult, RecordingLibrary, RepairRecordingGoal,
        SnapshotRecordingGoal, StopRecordingGoal,
    },
};
use blueos_recorder_app::RecorderService;
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_service::{Service, new_job_id, testing::Harness};

use common::harness::io::drain_blocking_io;
use common::harness::jobs::{next_job_result, subscribe_job_results};
use common::harness::recording::start_recording;
use common::harness::startup::start_harness;
use common::harness::state::{
    wait_for_active_recording, wait_for_library_file_listed, wait_for_library_state,
    wait_for_recording_idle,
};

struct ListedDeleteFixture {
    directory: TempDir,
    victim: PathBuf,
    harness: Harness<RecorderService>,
    results: Subscriber,
}

async fn listed_delete_fixture() -> ListedDeleteFixture {
    let directory = tempdir().expect("tempdir");
    let victim = directory.path().join("finished.mcap");
    fs::write(&victim, b"data").expect("write");
    let harness = start_harness(directory.path()).await;
    let results = subscribe_job_results(&harness, "DeleteRecording").await;
    wait_for_library_file_listed(&harness, "finished.mcap").await;
    ListedDeleteFixture {
        directory,
        victim,
        harness,
        results,
    }
}

async fn stop_auto_recording_and_remove_session_files(
    harness: &Harness<RecorderService>,
    directory: &std::path::Path,
) {
    start_recording(harness).await;
    advance(Duration::from_secs(1)).await;
    wait_for_active_recording(harness.backend()).await;
    harness
        .send("Stop", &StopRecordingGoal::default())
        .await
        .unwrap();
    wait_for_recording_idle(harness.backend()).await;
    for entry in fs::read_dir(directory).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("recorder_") && name.ends_with(".mcap") {
            let _ = fs::remove_file(entry.path());
        }
    }
    advance(RESCAN_INTERVAL).await;
    wait_for_library_state(harness.backend(), |library| {
        !library
            .files
            .iter()
            .any(|file| file.path.starts_with("recorder_"))
    })
    .await;
}

async fn drain_subscriber(updates: &mut blueos_comms::Subscriber) {
    while timeout(Duration::from_millis(10), updates.recv())
        .await
        .is_ok()
    {}
}

#[tokio::test(start_paused = true)]
async fn unchanged_rescan_does_not_republish_library_state() {
    let directory = tempdir().expect("tempdir");
    fs::write(directory.path().join("finished.mcap"), b"not a real mcap").expect("write");

    let harness = start_harness(directory.path()).await;
    let mut updates = harness
        .backend()
        .subscribe(&state_key(RecorderService::NAME, "library"))
        .await
        .expect("subscribe");

    stop_auto_recording_and_remove_session_files(&harness, directory.path()).await;
    wait_for_library_file_listed(&harness, "finished.mcap").await;
    drain_subscriber(&mut updates).await;
    let before = harness.state::<RecordingLibrary>("library").await.unwrap();

    advance(RESCAN_INTERVAL + Duration::from_secs(1)).await;
    drain_blocking_io().await;

    let after = harness.state::<RecordingLibrary>("library").await.unwrap();
    assert_eq!(before, after, "catalog must be unchanged after rescan");
    let second = timeout(Duration::from_millis(100), updates.recv()).await;
    assert!(
        second.is_err(),
        "unchanged catalog must not publish again (Kernel dedupes identical State payloads)"
    );
}

#[tokio::test(start_paused = true)]
async fn each_goal_with_an_invalid_path_is_rejected_in_the_ack_without_touching_disk() {
    let directory = tempdir().expect("tempdir");
    let victim = directory.path().join("safe.mcap");
    fs::write(&victim, b"data").expect("write");

    let harness = start_harness(directory.path()).await;

    for (path, reason) in [
        ("../outside.mcap", "Invalid recording path."),
        ("/etc/passwd.mcap", "Invalid recording path."),
        ("notes.txt", "Only .mcap recordings are supported."),
    ] {
        let path = path.to_owned();
        let acks = [
            harness
                .send(
                    "DeleteRecording",
                    &DeleteRecordingGoal { path: path.clone() },
                )
                .await
                .unwrap(),
            harness
                .send(
                    "RepairRecording",
                    &RepairRecordingGoal { path: path.clone() },
                )
                .await
                .unwrap(),
            harness
                .send(
                    "SnapshotRecording",
                    &SnapshotRecordingGoal { path: path.clone() },
                )
                .await
                .unwrap(),
        ];
        for ack in acks {
            assert!(!ack.accepted, "expected a rejection for {path:?}");
            assert_eq!(ack.reason, reason, "{path:?}");
        }
    }
    assert!(
        harness.jobs().await.unwrap().jobs.is_empty(),
        "a rejected Goal must not create a Job"
    );
    assert!(
        victim.exists(),
        "a rejected delete must not remove library files"
    );
}

#[tokio::test(start_paused = true)]
async fn a_delete_succeeds_once_the_file_is_gone_and_names_it_in_its_job_result() {
    let ListedDeleteFixture {
        directory: _directory,
        victim,
        harness,
        mut results,
    } = listed_delete_fixture().await;

    let job_id = new_job_id();
    let ack = harness
        .submit(
            "DeleteRecording",
            job_id,
            &DeleteRecordingGoal {
                path: "finished.mcap".into(),
            },
        )
        .await
        .unwrap();
    let (job, result) = next_job_result::<DeleteRecordingResult>(&mut results).await;

    assert_eq!(ack.status, CommandAckStatus::Executing);
    assert_eq!(
        (job.job_id, job.status, job.reason.as_str()),
        (job_id.to_string(), JobStatusStatus::Succeeded, "")
    );
    assert_eq!(result.path, "finished.mcap");
    assert!(!victim.exists());
}

#[tokio::test(start_paused = true)]
async fn a_delete_that_cannot_remove_the_file_aborts_its_job_with_the_error() {
    let ListedDeleteFixture {
        directory: _directory,
        victim,
        harness,
        mut results,
    } = listed_delete_fixture().await;
    fs::remove_file(&victim).expect("remove the recording");
    fs::create_dir(&victim).expect("put a folder where the recording was");

    let job_id = new_job_id();
    harness
        .submit(
            "DeleteRecording",
            job_id,
            &DeleteRecordingGoal {
                path: "finished.mcap".into(),
            },
        )
        .await
        .unwrap();
    let (job, result) = next_job_result::<DeleteRecordingResult>(&mut results).await;

    assert_eq!(
        (job.job_id, job.status, job.reason.as_str()),
        (
            job_id.to_string(),
            JobStatusStatus::Aborted,
            "invalid recording path"
        )
    );
    assert_eq!(result.path, "finished.mcap");
    assert!(victim.is_dir(), "a failed delete leaves the disk as it was");
}
