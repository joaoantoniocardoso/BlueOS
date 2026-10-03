//! Recorder library State and delete (layer L3, paused clock).

mod common;

use core::time::Duration;
use std::{fs, sync::Arc};

use tempfile::tempdir;
use tokio::time::{advance, timeout};

use blueos_api::state_key;
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_idl::msg::blueos_recorder_msgs::{
    DeleteRecordingCommand, RecordingLibrary, StopRecordingCommand,
};
use blueos_recorder_app::{RecorderArguments, RecorderService};
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_service::{Service, testing::Harness};

use common::{
    drain_blocking_io, recorder_arguments, start_harness, wait_for_active_recording,
    wait_for_library_file_listed, wait_for_recording_idle,
};

#[tokio::test(start_paused = true)]
async fn unchanged_rescan_does_not_republish_library_state() {
    let directory = tempdir().expect("tempdir");
    fs::write(directory.path().join("finished.mcap"), b"not a real mcap").expect("write");

    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let state_key = state_key(RecorderService::NAME, "library");
    let mut updates = backend.subscribe(&state_key).await.expect("subscribe");

    let harness = Harness::start_on(Arc::clone(&backend), recorder_arguments(directory.path()))
        .await
        .expect("harness");

    stop_auto_recording_and_remove_session_files(&harness, directory.path()).await;
    wait_for_library_file_listed(&harness, "finished.mcap").await;
    drain_subscriber(&mut updates).await;
    let before = harness.state::<RecordingLibrary>("library").await;

    advance(RESCAN_INTERVAL + Duration::from_secs(1)).await;
    drain_blocking_io().await;

    let after = harness.state::<RecordingLibrary>("library").await;
    assert_eq!(before, after, "catalog must be unchanged after rescan");
    let second = timeout(Duration::from_millis(100), updates.recv()).await;
    assert!(
        second.is_err(),
        "unchanged catalog must not publish again (Kernel dedupes identical State payloads)"
    );
}

#[tokio::test(start_paused = true)]
async fn delete_rejects_hostile_paths_without_touching_disk() {
    let directory = tempdir().expect("tempdir");
    let victim = directory.path().join("safe.mcap");
    fs::write(&victim, b"data").expect("write");

    let harness = start_harness(directory.path()).await;

    for path in ["../outside.mcap", "/etc/passwd.mcap", "notes.txt"] {
        let ack = harness
            .send(
                "DeleteRecording",
                &DeleteRecordingCommand { path: path.into() },
            )
            .await;
        assert!(!ack.accepted, "expected refusal for {path:?}");
    }
    assert!(
        victim.exists(),
        "hostile delete must not remove library files"
    );
}

async fn stop_auto_recording_and_remove_session_files(
    harness: &Harness<RecorderService>,
    directory: &std::path::Path,
) {
    advance(Duration::from_secs(1)).await;
    wait_for_active_recording(harness.backend()).await;
    harness.send("Stop", &StopRecordingCommand::default()).await;
    wait_for_recording_idle(harness.backend()).await;
    for entry in fs::read_dir(directory).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("recorder_") && name.ends_with(".mcap") {
            let _ = fs::remove_file(entry.path());
        }
    }
    advance(RESCAN_INTERVAL).await;
    drain_blocking_io().await;
}

async fn drain_subscriber(updates: &mut blueos_comms::Subscriber) {
    while timeout(Duration::from_millis(10), updates.recv())
        .await
        .is_ok()
    {}
}
