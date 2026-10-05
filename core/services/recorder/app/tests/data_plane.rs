//! Recorder data plane integration tests (layer L3, paused clock).

mod common;

use core::time::Duration;
use std::fs;

use bytes::Bytes;
use tempfile::tempdir;
use tokio::time::advance;

use blueos_comms::{Payload, Sample};
use blueos_domain::Command;
use blueos_recorder_cameras::{CamerasObservedFact, RAW_MAVLINK_OUT_TOPIC, SystemAndComponent};
use blueos_recorder_capture::CaptureObservedFact;
use blueos_recorder_domain::{RecorderObservedFact, RecorderRequest};
use blueos_recorder_mavlink::{test_camera_capture_frame, test_vehicle_heartbeat_frame};

use common::harness::recording::{
    active_recording_mcap_path, publish_pump_state, start_recording,
    stop_recording_and_finalize_mcap,
};
use common::harness::startup::start_harness;
use common::harness::state::{
    wait_for_active_recording, wait_for_recording_bytes, wait_for_recording_state,
};
use common::mcap_fixtures::mcap_channel_topic_names;

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
    wait_for_recording_state(harness.backend(), |state| !state.armed).await;

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

    let topics = mcap_channel_topic_names(&path);
    assert!(!topics.iter().any(|topic| topic == "mavlink/1/1/HEARTBEAT"));
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

    let topics = mcap_channel_topic_names(&path);
    assert!(topics.iter().any(|topic| topic == "mavlink/1/1/HEARTBEAT"));
}

#[tokio::test(start_paused = true)]
async fn dropped_armed_fact_heals_on_mavlink_periodic_resend() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    harness
        .backend()
        .publish(Sample::new(
            RAW_MAVLINK_OUT_TOPIC,
            Payload::new(Bytes::from(test_vehicle_heartbeat_frame(true))),
            "application/octet-stream",
        ))
        .await
        .expect("publish armed heartbeat");
    wait_for_recording_state(harness.backend(), |state| state.armed).await;

    harness
        .command_sender()
        .send(Command::ObservedFact(RecorderObservedFact::Capture(
            CaptureObservedFact::ArmedChanged(false),
        )))
        .await
        .expect("stale disarmed fact");
    wait_for_recording_state(harness.backend(), |state| !state.armed).await;

    advance(Duration::from_secs(1)).await;
    wait_for_recording_state(harness.backend(), |state| state.armed).await;
}

#[tokio::test(start_paused = true)]
async fn the_always_on_recording_shows_the_armed_vehicle_in_the_recording_state_without_a_job() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    wait_for_active_recording(harness.backend()).await;

    harness
        .backend()
        .publish(Sample::new(
            RAW_MAVLINK_OUT_TOPIC,
            Payload::new(Bytes::from(test_vehicle_heartbeat_frame(true))),
            "application/octet-stream",
        ))
        .await
        .expect("publish armed heartbeat");
    wait_for_recording_state(harness.backend(), |state| {
        state.armed && state.session_active
    })
    .await;

    assert!(harness.jobs().await.unwrap().jobs.is_empty());
    assert!(harness.job_history("Start").await.unwrap().jobs.is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_mavlink_camera_start_and_stop_capture_record_its_video_topic_without_a_job() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    wait_for_active_recording(harness.backend()).await;
    let topic = "video/front/stream";
    let camera = SystemAndComponent {
        system_id: 1,
        component_id: 100,
    };
    for discovered in [
        CamerasObservedFact::SetCameraRecordingCapability {
            camera,
            capture_video: true,
        },
        CamerasObservedFact::RegisterVideoStream {
            topic: topic.into(),
            camera,
        },
    ] {
        harness
            .command_sender()
            .send(Command::ObservedFact(RecorderObservedFact::Cameras(
                discovered,
            )))
            .await
            .expect("camera discovered");
    }

    for (start, recording) in [(true, vec![topic.to_owned()]), (false, vec![])] {
        harness
            .backend()
            .publish(Sample::new(
                RAW_MAVLINK_OUT_TOPIC,
                Payload::new(Bytes::from(test_camera_capture_frame(start, 1, 100))),
                "application/octet-stream",
            ))
            .await
            .expect("publish capture command");
        wait_for_recording_state(harness.backend(), |state| {
            state.recording_video_topics == recording
        })
        .await;
    }

    assert!(harness.jobs().await.unwrap().jobs.is_empty());
}
