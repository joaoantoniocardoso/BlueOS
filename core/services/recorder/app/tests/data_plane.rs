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
    let path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &path).await;
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
    let path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &path).await;

    let data = fs::read(path).expect("read");
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

    let first_path = directory.path().join(&first_file_name);
    wait_for_mcap_finalized(&first_path).await;
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
    let path = active_recording_mcap_path_on(&backend, directory.path()).await;

    shutdown.trigger();
    timeout(REPLY_TIMEOUT, run)
        .await
        .expect("shutdown")
        .expect("join");

    assert_mcap_readable(&path).await;
    let bytes = fs::read(&path).expect("read");
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
    let path = active_recording_mcap_path_on(&backend, directory.path()).await;

    shutdown.trigger();
    timeout(REPLY_TIMEOUT, run)
        .await
        .expect("shutdown")
        .expect("join");

    assert_mcap_readable(&path).await;
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
    let path = active_recording_mcap_path_on(&backend, directory.path()).await;

    shutdown.trigger();
    timeout(REPLY_TIMEOUT, run)
        .await
        .expect("shutdown")
        .expect("join");

    assert_mcap_readable(&path).await;
    let bytes = fs::read(&path).expect("read");
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

async fn active_recording_mcap_path(
    harness: &Harness<RecorderService>,
    directory: &std::path::Path,
) -> std::path::PathBuf {
    active_recording_mcap_path_on(harness.backend(), directory).await
}

async fn active_recording_mcap_path_on(
    backend: &Arc<dyn CommsBackend>,
    directory: &std::path::Path,
) -> std::path::PathBuf {
    wait_for_active_recording(backend).await;
    directory.join(recording_state(backend).await.current_file)
}

/// Stop clears `recording` State before the writer finishes, so no State update marks finalize.
/// Wait for idle, then advance virtual time until the data plane Task completes `finish`.
async fn stop_recording_and_finalize_mcap(backend: &Arc<dyn CommsBackend>, path: &std::path::Path) {
    stop_recording_on(backend).await;
    wait_for_recording_idle(backend).await;
    wait_for_mcap_finalized(path).await;
}

async fn stop_recording_on(backend: &Arc<dyn CommsBackend>) {
    let stop = StopRecordingCommand::default();
    let body = QueryBody::new(
        stop.encode().expect("encode"),
        cdr_encoding(StopRecordingCommand::SCHEMA_NAME),
    );
    backend
        .get(
            &command_key(RecorderService::NAME, "Stop"),
            Some(body),
            REPLY_TIMEOUT,
        )
        .await
        .expect("stop");
}

async fn assert_mcap_readable(path: &std::path::Path) {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let bytes = fs::read(&path).expect("read mcap");
        mcap::Summary::read(&bytes)
            .expect("parse mcap")
            .expect("mcap summary");
    })
    .await
    .expect("read task");
}

async fn wait_for_mcap_finalized(path: &std::path::Path) {
    let path = path.to_path_buf();
    timeout(REPLY_TIMEOUT, async {
        loop {
            let finalized = tokio::task::spawn_blocking({
                let path = path.clone();
                move || -> Option<()> {
                    let bytes = fs::read(&path).ok()?;
                    mcap::Summary::read(&bytes).ok().flatten()?;
                    Some(())
                }
            })
            .await
            .ok()
            .flatten()
            .is_some();
            if finalized {
                return;
            }
            advance(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("mcap file never finalized");
}

async fn wait_for_recording_idle(backend: &Arc<dyn CommsBackend>) {
    wait_for_recording_state(backend, false, |state| !state.session_active).await;
}

async fn wait_for_active_recording(backend: &Arc<dyn CommsBackend>) {
    wait_for_recording_state(backend, false, |state| !state.current_file.is_empty()).await;
}

async fn wait_for_rotated_recording(backend: &Arc<dyn CommsBackend>, previous_file: &str) {
    let previous_file = previous_file.to_owned();
    wait_for_recording_state(backend, false, |state| {
        state.session_active
            && !state.current_file.is_empty()
            && state.current_file != previous_file
    })
    .await;
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
    wait_for_recording_state(backend, true, |state| {
        state.session_bytes_written >= minimum
    })
    .await;
}

async fn wait_for_recording_state(
    backend: &Arc<dyn CommsBackend>,
    advance_bytes_report_interval: bool,
    predicate: impl Fn(&RecordingState) -> bool,
) {
    let key = state_key(RecorderService::NAME, "recording");
    let mut updates = backend
        .subscribe(&key)
        .await
        .expect("subscribe recording state");

    if predicate(&recording_state(backend).await) {
        return;
    }

    if advance_bytes_report_interval {
        advance(Duration::from_secs(1)).await;
        if predicate(&recording_state(backend).await) {
            return;
        }
    }

    timeout(REPLY_TIMEOUT, async {
        while let Some(sample) = updates.recv().await {
            let state = RecordingState::decode(&sample.payload().to_bytes())
                .expect("decode recording state");
            if predicate(&state) {
                return;
            }
        }
        panic!("recording state subscription closed");
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for recording state"));
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
