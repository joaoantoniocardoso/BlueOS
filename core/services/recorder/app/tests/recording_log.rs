//! A recording's lifecycle is visible on the Recorder's `log` key (D-13).

mod common;

use core::time::Duration;
use std::sync::Arc;

use bytes::Bytes;
use tempfile::tempdir;
use tokio::time::timeout;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use blueos_api::{Message, log_key};
use blueos_comms::{Payload, Sample, Subscriber};
use blueos_domain::Command;
use blueos_idl::msg::foxglove_msgs::Log;
use blueos_logging::{attach, init};
use blueos_recorder_app::RecorderService;
use blueos_recorder_domain::RecorderRequest;
use blueos_service::Service;

use common::harness::recording::{
    active_recording_mcap_path, start_recording, stop_recording_and_finalize_mcap,
};
use common::harness::startup::start_harness;
use common::harness::state::{wait_for_active_recording, wait_for_recording_bytes};

const LOG_QUIET_PERIOD: Duration = Duration::from_secs(2);

#[tokio::test(start_paused = true)]
async fn recording_lifecycle_is_logged_on_the_log_key() {
    init(0);
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    let key = log_key(RecorderService::NAME);
    let mut subscriber = harness.backend().subscribe(&key).await.expect("subscribe");
    let publisher = attach(Arc::clone(harness.backend()), key).await;
    let tasks = TaskTracker::new();
    tasks.spawn(publisher.run(CancellationToken::new()));

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
    wait_for_recording_bytes(&harness, 1).await;
    let path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &path).await;

    let messages = log_messages(&mut subscriber).await;
    for expected in [
        "Opened a recording",
        "Started recording a video stream",
        "Added a channel to the recording",
        "Finished a recording",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "expected a `{expected}` line in {messages:#?}"
        );
    }
}

async fn log_messages(subscriber: &mut Subscriber) -> Vec<String> {
    let mut messages = Vec::new();
    while let Ok(Some(sample)) = timeout(LOG_QUIET_PERIOD, subscriber.recv()).await {
        let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
        messages.push(decoded.message);
    }
    messages
}
