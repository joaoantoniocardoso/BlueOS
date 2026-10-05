//! ROS 2 schema gate integration tests (layer L3, paused clock).

mod common;

use core::time::Duration;
use std::{fs, sync::Arc};

use bytes::Bytes;
use tempfile::tempdir;
use tokio::time::advance;

use blueos_comms::{CommsBackend, LivelinessToken, Payload, Sample};

use common::harness::recording::{
    active_recording_mcap_path, start_recording, stop_recording_and_finalize_mcap,
};
use common::harness::startup::start_harness;
use common::harness::state::{
    wait_for_active_recording, wait_for_recording_bytes, wait_for_recording_idle,
};
use common::mcap_fixtures::recorder_mcap_paths;

const ZENOH_ID: &str = "aac3178e146ba6f1fc6e6a4085e77f21";
const CDR_SAMPLE: [u8; 4] = [0x00, 0x01, 0x00, 0x00];
const RMW_CHATTER_KEY: &str = "0/chatter/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18";

fn ros2dds_token(data_key: &str) -> String {
    let escaped_key = data_key.replace('/', "\u{a7}");
    format!("@/{ZENOH_ID}/@ros2_lv/MP/{escaped_key}/std_msgs\u{a7}msg\u{a7}String")
}

fn mcap_counts(path: &std::path::Path) -> (usize, usize, usize) {
    let bytes = fs::read(path).expect("read mcap");
    let summary = mcap::read::Summary::read(&bytes)
        .expect("summary")
        .expect("summary section");
    (
        summary.channels.len(),
        summary.schemas.len(),
        mcap::MessageStream::new(&bytes).expect("messages").count(),
    )
}

#[tokio::test(start_paused = true)]
async fn ros2dds_sample_before_token_is_written_with_schema_when_token_arrives_within_two_seconds()
{
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    harness
        .backend()
        .publish(Sample::new(
            "chatter",
            Payload::new(Bytes::from_static(&CDR_SAMPLE)),
            "zenoh/bytes",
        ))
        .await
        .expect("publish");

    let _token = declare_liveliness(harness.backend(), &ros2dds_token("chatter")).await;
    advance(Duration::from_millis(500)).await;
    wait_for_recording_bytes(&harness, 1).await;

    let path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &path).await;

    let (channels, schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 1);
    assert_eq!(schemas, 1);
    assert_eq!(messages, 1);
}

#[tokio::test(start_paused = true)]
async fn ros2dds_timeout_produces_schema_less_channel_then_schema_channel_on_same_topic() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    harness
        .backend()
        .publish(Sample::new(
            "chatter",
            Payload::new(Bytes::from_static(&CDR_SAMPLE)),
            "zenoh/bytes",
        ))
        .await
        .expect("publish");

    advance(Duration::from_secs(3)).await;
    wait_for_recording_bytes(&harness, 1).await;

    let _token = declare_liveliness(harness.backend(), &ros2dds_token("chatter")).await;
    advance(Duration::from_millis(200)).await;

    harness
        .backend()
        .publish(Sample::new(
            "chatter",
            Payload::new(Bytes::from_static(&CDR_SAMPLE)),
            "zenoh/bytes",
        ))
        .await
        .expect("publish second");

    advance(Duration::from_millis(500)).await;
    wait_for_recording_bytes(&harness, 2).await;

    let path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &path).await;

    let (channels, schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 2);
    assert_eq!(schemas, 1);
    assert_eq!(messages, 2);
}

#[tokio::test(start_paused = true)]
async fn unknown_ros2_type_is_recorded_on_fallback_channel() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    let unknown_token =
        format!("@/{ZENOH_ID}/@ros2_lv/MP/chatter/unknown_pkg\u{a7}msg\u{a7}NotInCatalog");
    let _token = declare_liveliness(harness.backend(), &unknown_token).await;
    advance(Duration::from_millis(50)).await;

    harness
        .backend()
        .publish(Sample::new(
            "chatter",
            Payload::new(Bytes::from_static(&CDR_SAMPLE)),
            "zenoh/bytes",
        ))
        .await
        .expect("publish");

    advance(Duration::from_millis(500)).await;
    wait_for_recording_bytes(&harness, 1).await;

    let path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &path).await;

    let (channels, _schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 1);
    assert_eq!(messages, 1);
}

#[tokio::test(start_paused = true)]
async fn held_ros2dds_sample_is_not_written_into_the_next_recording() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    harness
        .backend()
        .publish(Sample::new(
            "chatter",
            Payload::new(Bytes::from_static(&[0x00, 0x01, 0x00, 0x00, 0xAA])),
            "zenoh/bytes",
        ))
        .await
        .expect("publish first session");

    advance(Duration::from_millis(500)).await;
    wait_for_recording_bytes(&harness, 1).await;
    let first_path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &first_path).await;
    wait_for_recording_idle(harness.backend()).await;

    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;
    let _token = declare_liveliness(harness.backend(), &ros2dds_token("chatter")).await;
    harness
        .backend()
        .publish(Sample::new(
            "chatter",
            Payload::new(Bytes::from_static(&[0x00, 0x01, 0x00, 0x00, 0xBB])),
            "zenoh/bytes",
        ))
        .await
        .expect("publish second session");
    advance(Duration::from_millis(500)).await;
    wait_for_recording_bytes(&harness, 1).await;
    let second_path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &second_path).await;

    let paths = recorder_mcap_paths(directory.path());
    assert_eq!(paths.len(), 2);
    let (_, _, first_messages) = mcap_counts(&paths[0]);
    let (_, _, second_messages) = mcap_counts(&paths[1]);
    assert_eq!(first_messages, 1);
    assert_eq!(
        second_messages, 1,
        "gate state from the first recording must not flush into the second file"
    );
}

#[tokio::test(start_paused = true)]
async fn rmw_zenoh_data_key_resolves_schema_without_gate() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    start_recording(&harness).await;
    wait_for_active_recording(harness.backend()).await;

    harness
        .backend()
        .publish(Sample::new(
            RMW_CHATTER_KEY,
            Payload::new(Bytes::from_static(&CDR_SAMPLE)),
            "zenoh/bytes",
        ))
        .await
        .expect("publish");

    advance(Duration::from_millis(200)).await;
    wait_for_recording_bytes(&harness, 1).await;

    let path = active_recording_mcap_path(&harness, directory.path()).await;
    stop_recording_and_finalize_mcap(harness.backend(), &path).await;

    let (channels, schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 1);
    assert_eq!(schemas, 1);
    assert_eq!(messages, 1);
}

async fn declare_liveliness(backend: &Arc<dyn CommsBackend>, key: &str) -> LivelinessToken {
    backend
        .declare_liveliness(key)
        .await
        .expect("declare liveliness")
}
