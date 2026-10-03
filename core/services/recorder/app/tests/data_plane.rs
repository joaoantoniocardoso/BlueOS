//! Recorder data plane integration tests (layer L3, paused clock).

mod common;

use core::time::Duration;
use std::{fs, sync::Arc};

use bytes::Bytes;
use tempfile::tempdir;
use tokio::time::{advance, timeout};

use blueos_comms::{CommsBackend, Payload, Sample, channel::ChannelBackend};
use blueos_domain::Command;
use blueos_idl::msg::{
    blueos_example_msgs::EmptyRequest,
    blueos_msgs::ServiceInfo,
    blueos_recorder_msgs::{RecordingState, StartRecordingCommand},
};
use blueos_recorder_app::{
    IndexQuerySetup, RecorderService, build_with_record_gate, build_with_record_gate_and_index,
};
use blueos_recorder_cameras::RAW_MAVLINK_OUT_TOPIC;
use blueos_recorder_capture::{CaptureObservedFact, RecordGate};
use blueos_recorder_domain::{RecorderObservedFact, RecorderRequest};
use blueos_recorder_mavlink::test_vehicle_heartbeat_frame;
use blueos_service::{Kernel, Service, ServiceContext, testing::PausedClock};

use blueos_recorder_library::RESCAN_INTERVAL;
use common::{
    REPLY_TIMEOUT, active_recording_mcap_path, active_recording_mcap_path_on, assert_mcap_readable,
    drain_blocking_io, publish_pump_state, recorder_arguments, recorder_mcaps, start_harness,
    start_recording, start_recording_on, stop_recording_and_finalize_mcap,
    wait_for_active_recording, wait_for_library_file_ready_by_name, wait_for_recording_bytes,
    wait_for_recording_bytes_on, wait_for_rotated_recording,
};

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
        .command_sender()
        .send(Command::Request(RecorderRequest::StartVideoRecording {
            topic: "video/camera1/stream".into(),
        }))
        .await
        .expect("start video");

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
async fn mavlink_not_recorded_while_disarmed_when_policy_enabled() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    harness
        .backend()
        .publish(Sample::new(
            "mavlink/1/1/HEARTBEAT",
            Payload::new(Bytes::from_static(b"mavlink-bytes")),
            "application/octet-stream",
        ))
        .await
        .expect("publish mavlink");
    advance(Duration::from_secs(1)).await;

    let path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &path).await;

    let data = fs::read(&path).expect("read");
    let summary = mcap::Summary::read(&data).expect("read").expect("summary");
    let topics: Vec<_> = summary
        .channels
        .values()
        .map(|channel| channel.topic.as_str())
        .collect();
    assert!(!topics.contains(&"mavlink/1/1/HEARTBEAT"));
}

#[tokio::test(start_paused = true)]
async fn mavlink_recorded_after_armed_observed_fact() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    harness
        .command_sender()
        .send(Command::ObservedFact(RecorderObservedFact::Capture(
            CaptureObservedFact::ArmedChanged(true),
        )))
        .await
        .expect("armed fact");

    harness
        .backend()
        .publish(Sample::new(
            "mavlink/1/1/HEARTBEAT",
            Payload::new(Bytes::from_static(b"mavlink-armed")),
            "application/octet-stream",
        ))
        .await
        .expect("publish mavlink");
    advance(Duration::from_secs(1)).await;

    let path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &path).await;

    let data = fs::read(&path).expect("read");
    let summary = mcap::Summary::read(&data).expect("read").expect("summary");
    let topics: Vec<_> = summary
        .channels
        .values()
        .map(|channel| channel.topic.as_str())
        .collect();
    assert!(topics.contains(&"mavlink/1/1/HEARTBEAT"));
}

#[tokio::test(start_paused = true)]
async fn dropped_armed_fact_heals_on_mavlink_periodic_resend() {
    let directory = tempdir().expect("tempdir");
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let context = ServiceContext::new(recorder_arguments(directory.path()), Arc::clone(&backend));
    let (builder, recorder_context, mut record_gate) =
        build_with_record_gate(&context).expect("build");
    let kernel = Kernel::start(
        RecorderService::NAME,
        builder,
        recorder_context,
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    )
    .await
    .expect("kernel");
    let command_sender = kernel.command_sender().expect("command sender").clone();
    tokio::spawn(kernel.run());

    start_recording_on(&backend).await;
    wait_for_active_recording(&backend).await;

    backend
        .publish(Sample::new(
            RAW_MAVLINK_OUT_TOPIC,
            Payload::new(Bytes::from(test_vehicle_heartbeat_frame(true))),
            "application/octet-stream",
        ))
        .await
        .expect("publish armed heartbeat");
    wait_for_record_gate(&mut record_gate, |gate| gate.armed).await;

    command_sender
        .send(Command::ObservedFact(RecorderObservedFact::Capture(
            CaptureObservedFact::ArmedChanged(false),
        )))
        .await
        .expect("stale disarmed fact");
    wait_for_record_gate(&mut record_gate, |gate| !gate.armed).await;

    advance(Duration::from_secs(1)).await;
    wait_for_record_gate(&mut record_gate, |gate| gate.armed).await;
}

async fn wait_for_record_gate(
    record_gate: &mut tokio::sync::watch::Receiver<RecordGate>,
    predicate: impl Fn(&RecordGate) -> bool,
) {
    if predicate(&record_gate.borrow()) {
        return;
    }
    timeout(REPLY_TIMEOUT, async {
        while record_gate.changed().await.is_ok() {
            if predicate(&record_gate.borrow()) {
                return;
            }
        }
        panic!("record gate subscription closed");
    })
    .await
    .expect("timed out waiting for record gate");
}

#[tokio::test(start_paused = true)]
async fn recorder_service_info_is_published() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    let info = harness
        .query::<EmptyRequest, ServiceInfo>("info", &EmptyRequest::default())
        .await
        .expect("info query");
    assert_eq!(info.name, RecorderService::NAME);
    assert_eq!(info.version, RecorderService::VERSION);
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

#[tokio::test(start_paused = true)]
async fn shutdown_mid_recording_leaves_readable_file() {
    let directory = tempdir().expect("tempdir");
    let settings_parent = tempdir().expect("settings");
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let context = ServiceContext::with_settings_path(
        recorder_arguments(directory.path()),
        Some(settings_parent.path().to_path_buf()),
        Arc::clone(&backend),
    );
    let recorder_context = RecorderService::context(&context).expect("context");
    let mut builder = RecorderService::build(&context, &recorder_context).expect("build");
    let shutdown = builder.shutdown_handle();
    let kernel = Kernel::start(
        RecorderService::NAME,
        builder,
        recorder_context,
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
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let context = ServiceContext::with_settings_path(
        recorder_arguments(directory.path()),
        Some(settings_parent.path().to_path_buf()),
        Arc::clone(&backend),
    );
    let recorder_context = RecorderService::context(&context).expect("context");
    let mut builder = RecorderService::build(&context, &recorder_context).expect("build");
    let shutdown = builder.shutdown_handle();
    let kernel = Kernel::start(
        RecorderService::NAME,
        builder,
        recorder_context,
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
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let context = ServiceContext::with_settings_path(
        recorder_arguments(directory.path()),
        Some(settings_parent.path().to_path_buf()),
        Arc::clone(&backend),
    );
    let (mut builder, recorder_context, _) =
        build_with_record_gate_and_index(&context, IndexQuerySetup::default(), 2).expect("build");
    let shutdown = builder.shutdown_handle();
    let kernel = Kernel::start(
        RecorderService::NAME,
        builder,
        recorder_context,
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
