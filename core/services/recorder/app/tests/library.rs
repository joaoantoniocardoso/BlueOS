//! Recorder library State and delete (layer L3, paused clock).

use core::time::Duration;
use std::{fs, sync::Arc};

use tempfile::tempdir;
use tokio::time::{advance, timeout};

use blueos_api::state_key;
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_idl::msg::blueos_recorder_msgs::{
    DeleteRecordingCommand, RecordingLibrary, RecordingState, StopRecordingCommand,
};
use blueos_recorder_app::{RecorderArguments, RecorderService};
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_service::{Service, testing::Harness};

#[tokio::test(start_paused = true)]
async fn unchanged_rescan_does_not_republish_library_state() {
    let directory = tempdir().expect("tempdir");
    fs::write(directory.path().join("finished.mcap"), b"not a real mcap").expect("write");

    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let state_key = state_key(RecorderService::NAME, "library");
    let mut updates = backend.subscribe(&state_key).await.expect("subscribe");

    let harness = Harness::start_on(
        Arc::clone(&backend),
        RecorderArguments {
            recorder_path: directory.path().to_path_buf(),
            mcap_writer_queue_capacity: None,
        },
    )
    .await
    .expect("harness");

    stop_auto_recording_and_remove_session_files(&harness, directory.path()).await;
    wait_for_library_file(&harness, "finished.mcap").await;
    for _ in 0..20 {
        advance(Duration::from_millis(50)).await;
        drain_subscriber(&mut updates).await;
    }
    let before = harness.state::<RecordingLibrary>("library").await;

    advance(RESCAN_INTERVAL + Duration::from_secs(1)).await;
    drain_rescan_io(&harness).await;

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

    let harness = Harness::<RecorderService>::start(RecorderArguments {
        recorder_path: directory.path().to_path_buf(),
        mcap_writer_queue_capacity: None,
    })
    .await
    .expect("harness");

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
    let recording = harness.state::<RecordingState>("recording").await;
    if recording.session_active {
        harness.send("Stop", &StopRecordingCommand::default()).await;
        for _ in 0..200 {
            advance(Duration::from_millis(50)).await;
            if !harness
                .state::<RecordingState>("recording")
                .await
                .session_active
            {
                break;
            }
        }
    }
    for entry in fs::read_dir(directory).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("recorder_") && name.ends_with(".mcap") {
            let _ = fs::remove_file(entry.path());
        }
    }
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
    }
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

async fn drain_subscriber(updates: &mut blueos_comms::Subscriber) {
    while timeout(Duration::from_millis(10), updates.recv())
        .await
        .is_ok()
    {}
}

async fn drain_rescan_io(harness: &Harness<RecorderService>) {
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
        let _recording: RecordingState = harness.state("recording").await;
        let _library: RecordingLibrary = harness.state("library").await;
    }
}
