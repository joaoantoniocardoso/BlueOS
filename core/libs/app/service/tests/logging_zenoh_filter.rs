//! Trace-level logging does not publish `zenoh*` targets on the `log` key.

use core::time::Duration;
use std::sync::Arc;

use tokio_util::{sync::CancellationToken, task::TaskTracker};

use blueos_api::log_key;
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_idl::{Message, msg::foxglove_msgs::Log};
use blueos_logging::{attach, init};

#[tokio::test(start_paused = true)]
async fn zenoh_targets_are_not_published_at_trace_level() {
    init(2);
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let key = log_key("fixture");
    let mut subscriber = backend.subscribe(&key).await.expect("subscribe");
    let publisher = attach(Arc::clone(&backend), key).await;
    let shutdown = CancellationToken::new();
    let tasks = TaskTracker::new();
    tasks.spawn(publisher.run(shutdown));

    tracing::trace!(target: "zenoh::api::session", payload_bytes = 42, "write");
    tracing::trace!(target: "fixture::domain", detail = "visible", "domain trace");

    let sample = tokio::time::timeout(Duration::from_secs(1), subscriber.recv())
        .await
        .expect("one log sample arrives")
        .expect("sample");
    assert_eq!(sample.key(), log_key("fixture"));
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert!(decoded.message.contains("domain trace"));
    assert!(!decoded.message.contains("write"));
}
