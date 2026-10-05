//! Records logged just before publisher shutdown are still published.

#[path = "logging/common.rs"]
mod logging_common;

use core::time::Duration;

use blueos_idl::{Message, msg::foxglove_msgs::Log};

use logging_common::{recv_log_sample, start_log_publisher};

#[tokio::test(start_paused = true)]
async fn record_logged_before_shutdown_is_published() {
    let mut fixture = start_log_publisher("fixture", 0).await;

    tracing::warn!(phase = "shutdown", "Flushing the log publisher");

    fixture.shutdown.cancel();
    fixture.tasks.close();
    let _ = tokio::time::timeout(Duration::from_secs(1), fixture.tasks.wait()).await;

    let sample = recv_log_sample(&mut fixture.subscriber, "log sample arrives").await;
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert!(decoded.message.contains("Flushing the log publisher"));
    assert!(decoded.message.contains("phase=shutdown"));
}
