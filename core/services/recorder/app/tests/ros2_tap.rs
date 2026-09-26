use std::collections::BTreeSet;
use std::time::Duration;

use blueos_comms::{ChannelBackend, Payload, Session};
use blueos_recorder::tap;
use blueos_recorder_mcap::{McapSession, McapWriteConfig};
use blueos_recorder_policy::TapPolicy;
use bytes::Bytes;
use tokio::sync::{mpsc, watch};
use tokio::time;

const ZENOH_ID: &str = "aac3178e146ba6f1fc6e6a4085e77f21";
const ZENOH_BYTES: &str = "zenoh/bytes";
const CDR_SAMPLE: [u8; 4] = [0x00, 0x01, 0x00, 0x00];
const RMW_CHATTER_KEY: &str = "0/chatter/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18";

fn ros2dds_token(data_key: &str) -> String {
    let escaped_key = data_key.replace('/', "\u{a7}");
    format!("@/{ZENOH_ID}/@ros2_lv/MP/{escaped_key}/std_msgs\u{a7}msg\u{a7}String")
}

async fn run_tap(
    service_session: Session,
    path: &std::path::Path,
) -> (tokio::task::JoinHandle<()>, McapSession) {
    let (_policy_sender, policy_receiver) = watch::channel(TapPolicy {
        session_active: true,
        armed: true,
        record_mavlink_only_when_armed: true,
        recording_video_topics: BTreeSet::new(),
    });
    let mcap_session = McapSession::open(path, McapWriteConfig::default()).expect("mcap");
    let writer = mcap_session.writer();
    let (fact_sender, _fact_receiver) = mpsc::channel(8);
    let tap_task = tokio::spawn(async move {
        tap::run_data_plane(service_session, policy_receiver, writer, None, fact_sender).await;
    });
    time::sleep(Duration::from_millis(50)).await;
    (tap_task, mcap_session)
}

fn mcap_counts(path: &std::path::Path) -> (usize, usize, usize) {
    let bytes = std::fs::read(path).expect("read mcap");
    let summary = mcap::read::Summary::read(&bytes)
        .expect("summary")
        .expect("summary section");
    let channels = summary.channels.len();
    let schemas = summary.schemas.len();
    let messages = mcap::MessageStream::new(&bytes).expect("messages").count();
    (channels, schemas, messages)
}

fn first_channel_message_encoding(path: &std::path::Path) -> String {
    let bytes = std::fs::read(path).expect("read mcap");
    let summary = mcap::read::Summary::read(&bytes)
        .expect("summary")
        .expect("summary section");
    summary
        .channels
        .values()
        .next()
        .expect("channel")
        .message_encoding
        .clone()
}

#[tokio::test]
async fn ros2dds_token_before_sample_application_cdr() {
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("token_first.mcap");

    let token = client_session
        .declare_liveliness(&ros2dds_token("chatter"))
        .await
        .expect("token");
    time::sleep(Duration::from_millis(30)).await;

    let (_tap, mcap_session) = run_tap(service_session, &path).await;

    client_session
        .publish(
            "chatter",
            Payload::from_bytes(Bytes::from_static(&CDR_SAMPLE)),
            "application/cdr",
            None,
        )
        .await
        .expect("publish");

    time::sleep(Duration::from_millis(200)).await;
    drop(token);
    mcap_session.finish().expect("finish");

    let (channels, schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 1);
    assert_eq!(schemas, 1);
    assert_eq!(messages, 1);
    assert_eq!(first_channel_message_encoding(&path), "cdr");
}

#[tokio::test]
async fn ros2dds_token_before_sample_zenoh_bytes() {
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("token_first_zenoh.mcap");

    let token = client_session
        .declare_liveliness(&ros2dds_token("chatter"))
        .await
        .expect("token");
    time::sleep(Duration::from_millis(30)).await;

    let (_tap, mcap_session) = run_tap(service_session, &path).await;

    client_session
        .publish(
            "chatter",
            Payload::from_bytes(Bytes::from_static(&CDR_SAMPLE)),
            ZENOH_BYTES,
            None,
        )
        .await
        .expect("publish");

    time::sleep(Duration::from_millis(200)).await;
    drop(token);
    mcap_session.finish().expect("finish");

    let (channels, schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 1);
    assert_eq!(schemas, 1);
    assert_eq!(messages, 1);
    assert_eq!(first_channel_message_encoding(&path), "cdr");
}

#[tokio::test]
async fn ros2dds_sample_before_token_get_resolves() {
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("sample_first.mcap");

    let (_tap, mcap_session) = run_tap(service_session, &path).await;

    client_session
        .publish(
            "chatter",
            Payload::from_bytes(Bytes::from_static(&CDR_SAMPLE)),
            ZENOH_BYTES,
            None,
        )
        .await
        .expect("publish");

    let token = client_session
        .declare_liveliness(&ros2dds_token("chatter"))
        .await
        .expect("token");
    time::sleep(Duration::from_millis(800)).await;
    drop(token);
    mcap_session.finish().expect("finish");

    let (channels, schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 1);
    assert_eq!(schemas, 1);
    assert_eq!(messages, 1);
}

#[tokio::test]
async fn ros2dds_no_token_fallback() {
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("fallback.mcap");

    let (_tap, mcap_session) = run_tap(service_session, &path).await;

    client_session
        .publish(
            "chatter",
            Payload::from_bytes(Bytes::from_static(&CDR_SAMPLE)),
            ZENOH_BYTES,
            None,
        )
        .await
        .expect("publish");

    time::sleep(Duration::from_millis(800)).await;
    mcap_session.finish().expect("finish");

    let (channels, schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 1);
    assert_eq!(schemas, 0);
    assert_eq!(messages, 1);
    assert_eq!(first_channel_message_encoding(&path), "cdr");
}

#[tokio::test]
async fn ros2dds_fallback_records_every_later_sample() {
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("fallback_every_sample.mcap");

    let (_tap, mcap_session) = run_tap(service_session, &path).await;

    for _ in 0..3 {
        client_session
            .publish(
                "chatter",
                Payload::from_bytes(Bytes::from_static(&CDR_SAMPLE)),
                ZENOH_BYTES,
                None,
            )
            .await
            .expect("publish");
        time::sleep(Duration::from_millis(300)).await;
    }
    mcap_session.finish().expect("finish");

    let (channels, schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 1);
    assert_eq!(schemas, 0);
    assert_eq!(messages, 3);
}

#[tokio::test]
async fn ros2dds_token_after_fallback_second_channel() {
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("second_channel.mcap");

    let (_tap, mcap_session) = run_tap(service_session, &path).await;

    client_session
        .publish(
            "chatter",
            Payload::from_bytes(Bytes::from_static(&CDR_SAMPLE)),
            ZENOH_BYTES,
            None,
        )
        .await
        .expect("publish");
    time::sleep(Duration::from_millis(800)).await;

    let token = client_session
        .declare_liveliness(&ros2dds_token("chatter"))
        .await
        .expect("token");
    time::sleep(Duration::from_millis(100)).await;

    client_session
        .publish(
            "chatter",
            Payload::from_bytes(Bytes::from_static(&CDR_SAMPLE)),
            ZENOH_BYTES,
            None,
        )
        .await
        .expect("publish");

    time::sleep(Duration::from_millis(600)).await;
    drop(token);
    mcap_session.finish().expect("finish");

    let (channels, schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 2);
    assert_eq!(schemas, 1);
    assert_eq!(messages, 2);
}

#[tokio::test]
async fn zenoh_bytes_json_payload_not_gated() {
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("json_not_gated.mcap");

    let (_tap, mcap_session) = run_tap(service_session, &path).await;

    client_session
        .publish(
            "custom/topic",
            Payload::from_bytes(Bytes::from_static(br#"{"x":1}"#)),
            ZENOH_BYTES,
            None,
        )
        .await
        .expect("publish");

    time::sleep(Duration::from_millis(800)).await;
    mcap_session.finish().expect("finish");

    let (channels, _schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 0);
    assert_eq!(messages, 0);
}

#[tokio::test]
async fn rmw_zenoh_zenoh_bytes_resolves_schema_from_key() {
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("rmw.mcap");

    let (_tap, mcap_session) = run_tap(service_session, &path).await;

    client_session
        .publish(
            RMW_CHATTER_KEY,
            Payload::from_bytes(Bytes::from_static(&CDR_SAMPLE)),
            ZENOH_BYTES,
            None,
        )
        .await
        .expect("publish");

    time::sleep(Duration::from_millis(200)).await;
    mcap_session.finish().expect("finish");

    let (channels, schemas, messages) = mcap_counts(&path);
    assert_eq!(channels, 1);
    assert_eq!(schemas, 1);
    assert_eq!(messages, 1);
    assert_eq!(first_channel_message_encoding(&path), "cdr");
}
