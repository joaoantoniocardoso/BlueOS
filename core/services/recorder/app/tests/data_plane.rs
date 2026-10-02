//! Recorder data plane integration tests (layer L3, paused clock).

use core::time::Duration;
use std::{fs, sync::Arc};

use bytes::Bytes;
use tempfile::tempdir;
use tokio::time::{advance, timeout};

use blueos_api::{Message, cdr_encoding, command_key, state_key};
use blueos_comms::{CommsBackend, Payload, QueryBody, Sample, channel::ChannelBackend};
use blueos_idl::msg::{
    blueos_example_msgs::PumpState,
    blueos_recorder_msgs::{RecordingState, StartRecordingCommand, StopRecordingCommand},
};
use blueos_recorder_app::{RecorderArguments, RecorderService};
use blueos_service::{
    Kernel, Service, ServiceContext,
    testing::{Harness, PausedClock},
};

const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::test(start_paused = true)]
async fn recording_blueos_topics_produces_mcap_with_ros2msg_schema() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;
    publish_pump_state(harness.backend()).await;
    wait_for_recording_bytes(&harness, 1).await;
    stop_recording(&harness).await;
    wait_for_recording_idle(harness.backend()).await;
    wait_for_mcap_readable(single_recorder_mcap(directory.path()).as_path()).await;

    let path = single_recorder_mcap(directory.path());
    let data = fs::read(&path).expect("mcap file");
    let summary = mcap::Summary::read(&data).expect("read").expect("summary");
    assert!(
        summary
            .schemas
            .values()
            .any(|schema| schema.encoding == "ros2msg"),
        "expected ros2msg schema in MCAP"
    );
}

#[tokio::test(start_paused = true)]
async fn records_video_and_non_blueos_keys() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    harness
        .backend()
        .publish(Sample::new(
            "video/camera1/stream",
            Payload::new(Bytes::from_static(b"video-bytes")),
            "application/octet-stream",
        ))
        .await
        .expect("publish video");
    harness
        .backend()
        .publish(Sample::new(
            "external/ros/topic",
            Payload::new(Bytes::from_static(b"ros-bytes")),
            "application/octet-stream",
        ))
        .await
        .expect("publish external");
    wait_for_recording_bytes(&harness, 1).await;
    stop_recording(&harness).await;
    wait_for_recording_idle(harness.backend()).await;
    wait_for_mcap_readable(single_recorder_mcap(directory.path()).as_path()).await;

    let data = fs::read(single_recorder_mcap(directory.path())).expect("read");
    let summary = mcap::Summary::read(&data).expect("read").expect("summary");
    let topics: Vec<_> = summary
        .channels
        .values()
        .map(|channel| channel.topic.as_str())
        .collect();
    assert!(topics.contains(&"video/camera1/stream"));
    assert!(topics.contains(&"external/ros/topic"));
}

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
        .current_file;
    harness
        .send(
            "Start",
            &StartRecordingCommand {
                rotate_if_active: true,
            },
        )
        .await;
    wait_for_rotated_recording(harness.backend(), &first_file_name).await;
    publish_pump_state(harness.backend()).await;
    advance(Duration::from_secs(1)).await;

    let state = harness.state::<RecordingState>("recording").await;
    assert!(state.session_active);
    assert!(state.current_file.starts_with("recorder_"));

    stop_recording(&harness).await;
    wait_for_recording_idle(harness.backend()).await;
    for path in recorder_mcaps(directory.path()) {
        wait_for_mcap_readable(path.as_path()).await;
    }

    let paths = recorder_mcaps(directory.path());
    assert_eq!(paths.len(), 2);
    for path in paths {
        let bytes = fs::read(path).expect("read");
        mcap::Summary::read(&bytes).expect("read").expect("summary");
    }
}

#[tokio::test(start_paused = true)]
async fn second_start_on_same_folder_does_not_overwrite_first_file() {
    let directory = tempdir().expect("tempdir");
    let first = start_harness(directory.path()).await;
    start_recording(&first).await;
    wait_for_active_recording(first.backend()).await;
    publish_pump_state(first.backend()).await;
    wait_for_recording_bytes(&first, 1).await;
    stop_recording(&first).await;
    wait_for_recording_idle(first.backend()).await;
    let first_path = single_recorder_mcap(directory.path());
    wait_for_mcap_readable(first_path.as_path()).await;
    let first_bytes = fs::read(&first_path).expect("read first");
    drop(first);

    let second = start_harness(directory.path()).await;
    start_recording(&second).await;
    wait_for_active_recording(second.backend()).await;
    publish_pump_state(second.backend()).await;
    wait_for_recording_bytes(&second, 1).await;
    stop_recording(&second).await;
    wait_for_recording_idle(second.backend()).await;
    for path in recorder_mcaps(directory.path()) {
        wait_for_mcap_readable(path.as_path()).await;
    }

    assert_eq!(recorder_mcaps(directory.path()).len(), 2);
    assert_eq!(
        fs::read(&first_path).expect("first still readable"),
        first_bytes
    );
}

#[tokio::test(start_paused = true)]
async fn shutdown_mid_recording_leaves_readable_file() {
    let directory = tempdir().expect("tempdir");
    let settings_parent = tempdir().expect("settings");
    let context = ServiceContext::with_settings_path(
        recorder_arguments(directory.path(), None),
        Some(settings_parent.path().to_path_buf()),
    );
    let mut builder = RecorderService::build(&context).expect("build");
    let shutdown = builder.shutdown_handle();
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let kernel = Kernel::start(
        RecorderService::NAME,
        builder,
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    )
    .await
    .expect("kernel");
    let run = tokio::spawn(kernel.run());

    start_recording_on(&backend).await;
    wait_for_active_recording(&backend).await;
    publish_pump_state(&backend).await;
    wait_for_recording_bytes_on(&backend, 1).await;

    shutdown.trigger();
    timeout(REPLY_TIMEOUT, run)
        .await
        .expect("shutdown")
        .expect("join");

    let bytes = fs::read(single_recorder_mcap(directory.path())).expect("read");
    mcap::Summary::read(&bytes)
        .expect("read")
        .expect("readable after shutdown");
}

#[tokio::test(start_paused = true)]
async fn shutdown_with_no_samples_after_start_still_finishes_file() {
    let directory = tempdir().expect("tempdir");
    let settings_parent = tempdir().expect("settings");
    let context = ServiceContext::with_settings_path(
        recorder_arguments(directory.path(), None),
        Some(settings_parent.path().to_path_buf()),
    );
    let mut builder = RecorderService::build(&context).expect("build");
    let shutdown = builder.shutdown_handle();
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let kernel = Kernel::start(
        RecorderService::NAME,
        builder,
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    )
    .await
    .expect("kernel");
    let run = tokio::spawn(kernel.run());

    start_recording_on(&backend).await;
    wait_for_active_recording(&backend).await;
    advance(Duration::from_secs(1)).await;

    shutdown.trigger();
    timeout(REPLY_TIMEOUT, run)
        .await
        .expect("shutdown")
        .expect("join");

    let path = wait_for_one_recorder_mcap(directory.path()).await;
    let bytes = fs::read(&path).expect("read");
    mcap::Summary::read(&bytes)
        .expect("read")
        .expect("readable after quiet shutdown");
}

#[tokio::test(start_paused = true)]
async fn shutdown_with_full_writer_queue_finishes_file() {
    let directory = tempdir().expect("tempdir");
    let settings_parent = tempdir().expect("settings");
    let context = ServiceContext::with_settings_path(
        recorder_arguments(directory.path(), Some(2)),
        Some(settings_parent.path().to_path_buf()),
    );
    let mut builder = RecorderService::build(&context).expect("build");
    let shutdown = builder.shutdown_handle();
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let kernel = Kernel::start(
        RecorderService::NAME,
        builder,
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    )
    .await
    .expect("kernel");
    let run = tokio::spawn(kernel.run());

    start_recording_on(&backend).await;
    wait_for_active_recording(&backend).await;
    for _ in 0..64 {
        publish_pump_state(&backend).await;
    }

    shutdown.trigger();
    timeout(REPLY_TIMEOUT, run)
        .await
        .expect("shutdown")
        .expect("join");

    let bytes = fs::read(single_recorder_mcap(directory.path())).expect("read");
    mcap::Summary::read(&bytes)
        .expect("read")
        .expect("readable after back pressure");
}

fn recorder_arguments(
    path: &std::path::Path,
    mcap_writer_queue_capacity: Option<usize>,
) -> RecorderArguments {
    RecorderArguments {
        recorder_path: path.to_path_buf(),
        mcap_writer_queue_capacity,
    }
}

async fn start_harness(path: &std::path::Path) -> Harness<RecorderService> {
    Harness::start(recorder_arguments(path, None))
        .await
        .expect("harness")
}

async fn start_recording(harness: &Harness<RecorderService>) {
    start_recording_on(harness.backend()).await;
}

async fn start_recording_on(backend: &Arc<dyn CommsBackend>) {
    let start = StartRecordingCommand {
        rotate_if_active: false,
    };
    let body = QueryBody::new(
        start.encode().expect("encode"),
        cdr_encoding(StartRecordingCommand::SCHEMA_NAME),
    );
    backend
        .get(
            &command_key(RecorderService::NAME, "Start"),
            Some(body),
            REPLY_TIMEOUT,
        )
        .await
        .expect("start");
}

async fn stop_recording(harness: &Harness<RecorderService>) {
    harness.send("Stop", &StopRecordingCommand::default()).await;
}

async fn publish_pump_state(backend: &Arc<dyn CommsBackend>) {
    let message = PumpState::default();
    backend
        .publish(Sample::new(
            "blueos/v1/example/state/pump",
            Payload::new(Bytes::from(message.encode().expect("encode"))),
            cdr_encoding(PumpState::SCHEMA_NAME),
        ))
        .await
        .expect("publish");
}

async fn wait_for_mcap_readable(path: &std::path::Path) {
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        if mcap::Summary::read(&bytes)
            .ok()
            .and_then(|summary| summary)
            .is_some()
        {
            return;
        }
    }
    panic!("mcap file never became readable at {}", path.display());
}

async fn wait_for_recording_idle(backend: &Arc<dyn CommsBackend>) {
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
        let state = recording_state(backend).await;
        if !state.session_active {
            return;
        }
    }
    panic!("recording never became idle");
}

async fn wait_for_active_recording(backend: &Arc<dyn CommsBackend>) {
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
        let state = recording_state(backend).await;
        if !state.current_file.is_empty() {
            return;
        }
    }
    panic!("recording never became active");
}

async fn wait_for_rotated_recording(backend: &Arc<dyn CommsBackend>, previous_file: &str) {
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
        let state = recording_state(backend).await;
        if state.session_active
            && !state.current_file.is_empty()
            && state.current_file != previous_file
        {
            return;
        }
    }
    panic!("recording never rotated to a new MCAP file");
}

async fn wait_for_one_recorder_mcap(directory: &std::path::Path) -> std::path::PathBuf {
    for _ in 0..200 {
        advance(Duration::from_millis(50)).await;
        let paths = recorder_mcaps(directory);
        if paths.len() == 1 {
            wait_for_mcap_readable(paths[0].as_path()).await;
            return paths[0].clone();
        }
    }
    panic!("expected one recorder MCAP in {}", directory.display());
}

async fn wait_for_recording_bytes(harness: &Harness<RecorderService>, minimum: u64) {
    wait_for_recording_bytes_on(harness.backend(), minimum).await;
}

async fn recording_state(backend: &Arc<dyn CommsBackend>) -> RecordingState {
    let replies = backend
        .get(
            &state_key(RecorderService::NAME, "recording"),
            None,
            REPLY_TIMEOUT,
        )
        .await
        .expect("state");
    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one recording state");
    };
    RecordingState::decode(&reply.payload().to_bytes()).expect("decode recording state")
}

async fn wait_for_recording_bytes_on(backend: &Arc<dyn CommsBackend>, minimum: u64) {
    for _ in 0..20 {
        advance(Duration::from_secs(1)).await;
        let replies = backend
            .get(
                &state_key(RecorderService::NAME, "recording"),
                None,
                REPLY_TIMEOUT,
            )
            .await
            .expect("state");
        let [Ok(reply)] = replies.as_slice() else {
            panic!("expected one recording state");
        };
        let state =
            RecordingState::decode(&reply.payload().to_bytes()).expect("decode recording state");
        if state.session_bytes_written >= minimum {
            return;
        }
    }
    panic!("recording bytes never reached {minimum}");
}

fn recorder_mcaps(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(directory)
        .expect("read dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("recorder_") && name.ends_with(".mcap"))
        })
        .collect()
}

fn single_recorder_mcap(directory: &std::path::Path) -> std::path::PathBuf {
    let mut paths = recorder_mcaps(directory);
    assert_eq!(paths.len(), 1, "expected one recorder mcap");
    paths.pop().expect("path")
}
