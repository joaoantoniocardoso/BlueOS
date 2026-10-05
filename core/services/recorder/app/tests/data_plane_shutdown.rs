//! Recorder data plane shutdown and metrics tests.

mod common;

use core::time::Duration;
use std::{collections::BTreeMap, fs, sync::Arc};

use bytes::Bytes;
use tempfile::tempdir;
use tokio::time::{advance, timeout};

use blueos_api::{Message, cdr_encoding, state_key};
use blueos_comms::{CommsBackend, Payload, Sample};
use blueos_domain::Command;
use blueos_idl::msg::blueos_msgs::{MetricCounter, ServiceMetrics, SettingsEnvelope};
use blueos_recorder_capture::CaptureObservedFact;
use blueos_recorder_domain::{RecorderObservedFact, RecorderRequest};
use common::REPLY_TIMEOUT;
use common::harness::io::assert_mcap_readable;
use common::harness::recording::{
    active_recording_mcap_path, publish_pump_state, start_recording,
    stop_recording_and_finalize_mcap,
};
use common::harness::startup::{start_harness, start_harness_with};
use common::harness::state::{
    wait_for_active_recording, wait_for_recorder_state, wait_for_recording_bytes,
    wait_for_recording_state,
};

#[tokio::test(start_paused = true)]
async fn shutdown_mid_recording_leaves_readable_file() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;

    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;
    publish_pump_state(harness.backend()).await;
    wait_for_recording_bytes(&harness, 1).await;
    let path = active_recording_mcap_path(&harness, directory.path()).await;

    timeout(REPLY_TIMEOUT, harness.shutdown())
        .await
        .expect("shutdown");

    assert_mcap_readable(&path).await;
    let bytes = fs::read(&path).expect("read");
    mcap::Summary::read(&bytes)
        .expect("read")
        .expect("readable after shutdown");
}

#[tokio::test(start_paused = true)]
async fn shutdown_with_no_samples_after_start_still_finishes_file() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;

    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;
    advance(Duration::from_secs(1)).await;
    let path = active_recording_mcap_path(&harness, directory.path()).await;

    timeout(REPLY_TIMEOUT, harness.shutdown())
        .await
        .expect("shutdown");

    assert_mcap_readable(&path).await;
    let bytes = fs::read(&path).expect("read");
    mcap::Summary::read(&bytes)
        .expect("read")
        .expect("readable after quiet shutdown");
}

#[tokio::test(start_paused = true)]
async fn shutdown_with_full_writer_queue_finishes_file() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness_with(directory.path(), |context| {
        context.mcap_writer_queue_bytes = 64;
    })
    .await;

    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;
    for _ in 0..64 {
        publish_pump_state(harness.backend()).await;
    }
    let path = active_recording_mcap_path(&harness, directory.path()).await;

    timeout(REPLY_TIMEOUT, harness.shutdown())
        .await
        .expect("shutdown");

    assert_mcap_readable(&path).await;
    let bytes = fs::read(&path).expect("read");
    mcap::Summary::read(&bytes)
        .expect("read")
        .expect("readable after back pressure");
}

#[tokio::test(start_paused = true)]
async fn samples_past_the_writer_queue_byte_budget_are_counted_in_the_recording_state() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness_with(directory.path(), |context| {
        context.mcap_writer_queue_bytes = 1024;
    })
    .await;

    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;
    for _ in 0..4 {
        harness
            .backend()
            .publish(Sample::new(
                "load/flood",
                Payload::new(Bytes::from(vec![0_u8; 4 * 1024])),
                "application/octet-stream",
            ))
            .await
            .expect("publish");
    }
    advance(Duration::from_secs(1)).await;

    wait_for_recording_state(harness.backend(), |state| state.samples_dropped >= 4).await;
}

async fn wait_for_metrics(
    backend: &Arc<dyn CommsBackend>,
    predicate: impl Fn(&ServiceMetrics) -> bool,
) {
    wait_for_recorder_state(backend, "metrics", predicate).await;
}

/// The value of the counter `name` of `lane` in `metrics`; zero when it is not listed.
fn lane_counter(metrics: &ServiceMetrics, name: &str, lane: &str) -> u64 {
    metrics
        .counters
        .iter()
        .find(|counter| {
            counter.name == name
                && counter
                    .labels
                    .iter()
                    .any(|label| label.name == "lane" && label.value == lane)
        })
        .map_or(0, |counter| counter.value)
}

#[tokio::test(start_paused = true)]
async fn written_samples_and_bytes_are_counted_per_lane_in_the_metrics_state() {
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
        .command_sender()
        .send(Command::ObservedFact(RecorderObservedFact::Capture(
            CaptureObservedFact::ArmedChanged(true),
        )))
        .await
        .expect("armed fact");

    for (topic, payload_bytes, samples) in [
        ("mavlink/1/1/HEARTBEAT", 50, 1),
        ("video/camera1/stream", 200, 2),
        ("load/flood", 100, 3),
    ] {
        for _ in 0..samples {
            harness
                .backend()
                .publish(Sample::new(
                    topic,
                    Payload::new(Bytes::from(vec![0_u8; payload_bytes])),
                    "application/octet-stream",
                ))
                .await
                .expect("publish");
        }
    }
    advance(Duration::from_secs(1)).await;

    wait_for_metrics(harness.backend(), |metrics| {
        [("mavlink", 50, 1), ("video", 400, 2), ("other", 300, 3)]
            .into_iter()
            .all(|(lane, bytes, samples)| {
                lane_counter(metrics, "bytes_written", lane) == bytes
                    && lane_counter(metrics, "samples_written", lane) == samples
                    && lane_counter(metrics, "samples_dropped", lane) == 0
            })
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn samples_past_the_writer_queue_byte_budget_are_counted_in_the_metrics_state() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness_with(directory.path(), |context| {
        context.mcap_writer_queue_bytes = 1024;
    })
    .await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;
    for _ in 0..4 {
        harness
            .backend()
            .publish(Sample::new(
                "load/flood",
                Payload::new(Bytes::from(vec![0_u8; 4 * 1024])),
                "application/octet-stream",
            ))
            .await
            .expect("publish");
    }
    advance(Duration::from_secs(1)).await;

    wait_for_metrics(harness.backend(), |metrics| {
        lane_counter(metrics, "samples_dropped", "other") == 4
            && lane_counter(metrics, "samples_dropped", "video") == 0
            && lane_counter(metrics, "samples_dropped", "mavlink") == 0
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn the_metrics_state_of_another_service_is_recorded_into_the_mcap_file() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;
    let published = ServiceMetrics {
        counters: vec![MetricCounter {
            name: "restarts".into(),
            labels: vec![],
            value: 7,
        }],
        ..ServiceMetrics::default()
    };
    harness
        .backend()
        .publish(Sample::new(
            state_key("example", "metrics"),
            Payload::new(Bytes::from(published.encode().expect("encode"))),
            cdr_encoding(ServiceMetrics::SCHEMA_NAME),
        ))
        .await
        .expect("publish metrics");
    publish_pump_state(harness.backend()).await;
    wait_for_recording_bytes(&harness, 1).await;
    let path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &path).await;

    let data = fs::read(&path).expect("read");
    let messages: BTreeMap<_, _> = mcap::MessageStream::new(&data)
        .expect("stream")
        .map(|message| message.expect("message"))
        .map(|message| (message.channel.topic.clone(), message))
        .collect();
    let metrics = &messages[&state_key("example", "metrics")];
    assert_eq!(
        ServiceMetrics::decode(&metrics.data).expect("decode"),
        published
    );
    assert_eq!(
        metrics.channel.schema.as_ref().expect("schema").name,
        ServiceMetrics::SCHEMA_NAME
    );
    assert_eq!(
        metrics.log_time,
        messages[&state_key("example", "pump")].log_time
    );
}

#[tokio::test(start_paused = true)]
async fn update_settings_command_applies_capture_settings() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    let envelope = SettingsEnvelope {
        document_json:
            r#"{"VERSION":1,"record_mavlink_only_when_armed":false,"auto_start_recording":false}"#
                .into(),
        fields: Vec::new(),
    };
    let ack = harness.send("UpdateSettings", &envelope).await.unwrap();
    assert!(ack.accepted);
}
