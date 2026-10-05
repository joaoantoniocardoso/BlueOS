//! Panics routed through tracing reach the service `log` key.

#[path = "logging/common.rs"]
mod logging_common;

use blueos_idl::{Message, msg::foxglove_msgs::Log};

use logging_common::{recv_log_sample, start_log_publisher};

#[tokio::test(start_paused = true)]
async fn panic_message_reaches_log_key() {
    let mut fixture = start_log_publisher("fixture", 0).await;

    let panic_result = std::panic::catch_unwind(|| {
        panic!("inventory mismatch");
    });
    assert!(panic_result.is_err());

    let sample = recv_log_sample(&mut fixture.subscriber, "panic log arrives").await;
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert!(decoded.message.contains("inventory mismatch"));
}
