//! Recorder data plane integration tests (rotation, shutdown, metrics).

//! Recorder data plane integration tests (layer L3, paused clock).

mod common;

use core::time::Duration;
use std::fs;

use tempfile::tempdir;
use tokio::time::advance;

use blueos_idl::msg::blueos_recorder_msgs::{RecordingState, StartRecordingGoal};
use blueos_recorder_library::RESCAN_INTERVAL;

use common::harness::io::{assert_mcap_readable, drain_blocking_io};
use common::harness::recording::{
    active_recording_mcap_path, publish_pump_state, start_recording,
    stop_recording_and_finalize_mcap,
};
use common::harness::startup::start_harness;
use common::harness::state::{
    wait_for_active_recording, wait_for_library_file_ready_by_name, wait_for_recording_bytes,
    wait_for_rotated_recording,
};
use common::mcap_fixtures::recorder_mcaps;

#[tokio::test(start_paused = true)]
async fn rotation_keeps_both_files_intact_and_state_correct() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;
    publish_pump_state(harness.backend()).await;
    wait_for_recording_bytes(&harness, 1).await;

    advance(Duration::from_secs(1)).await;
    let first_file_name = harness
        .state::<RecordingState>("recording")
        .await
        .unwrap()
        .current_file;
    harness
        .send(
            "Start",
            &StartRecordingGoal {
                rotate_if_active: true,
            },
        )
        .await
        .unwrap();
    wait_for_rotated_recording(harness.backend(), &first_file_name).await;
    publish_pump_state(harness.backend()).await;
    advance(Duration::from_secs(1)).await;

    let state = harness.state::<RecordingState>("recording").await.unwrap();
    assert!(state.session_active);
    assert!(state.current_file.starts_with("recorder_"));

    advance(RESCAN_INTERVAL).await;
    drain_blocking_io().await;
    wait_for_library_file_ready_by_name(harness.backend(), &first_file_name).await;
    let first_path = directory.path().join(&first_file_name);
    assert_mcap_readable(&first_path).await;
    let second_path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &second_path).await;
    assert_mcap_readable(&second_path).await;

    let paths = recorder_mcaps(directory.path());
    assert_eq!(paths.len(), 2);
}

#[tokio::test(start_paused = true)]
async fn second_start_on_same_folder_does_not_overwrite_first_file() {
    let directory = tempdir().expect("tempdir");
    let first = start_harness(directory.path()).await;
    start_recording(&first).await;
    wait_for_active_recording(first.backend()).await;
    publish_pump_state(first.backend()).await;
    wait_for_recording_bytes(&first, 1).await;
    let first_path = active_recording_mcap_path(&first, directory.path()).await;
    stop_recording_and_finalize_mcap(first.backend(), &first_path).await;
    let first_bytes = fs::read(&first_path).expect("read first");
    drop(first);

    let second = start_harness(directory.path()).await;
    start_recording(&second).await;
    wait_for_active_recording(second.backend()).await;
    publish_pump_state(second.backend()).await;
    wait_for_recording_bytes(&second, 1).await;
    let second_path = active_recording_mcap_path(&second, directory.path()).await;
    stop_recording_and_finalize_mcap(second.backend(), &second_path).await;
    assert_mcap_readable(&first_path).await;
    assert_mcap_readable(&second_path).await;

    assert_eq!(recorder_mcaps(directory.path()).len(), 2);
    assert_eq!(
        fs::read(&first_path).expect("first still readable"),
        first_bytes
    );
}
