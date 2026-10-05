//! Trace-level logging does not publish `zenoh*` targets on the `log` key.

#[path = "logging/common.rs"]
mod logging_common;

use blueos_api::log_key;
use blueos_idl::{Message, msg::foxglove_msgs::Log};

use logging_common::{recv_log_sample, start_log_publisher};

#[tokio::test(start_paused = true)]
async fn zenoh_targets_are_not_published_at_trace_level() {
    let mut fixture = start_log_publisher("fixture", 2).await;

    tracing::trace!(target: "zenoh::api::session", payload_bytes = 42, "write");
    tracing::trace!(target: "fixture::domain", detail = "visible", "domain trace");

    let sample = recv_log_sample(&mut fixture.subscriber, "one log sample arrives").await;
    assert_eq!(sample.key(), log_key("fixture"));
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert!(decoded.message.contains("domain trace"));
    assert!(!decoded.message.contains("write"));
}
