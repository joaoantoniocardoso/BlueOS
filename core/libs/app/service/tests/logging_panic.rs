//! Panics routed through tracing reach the service `log` key.

use std::{sync::Arc, time::Duration};

use blueos_api::log_key;
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_idl::{Message, msg::foxglove_msgs::Log};
use blueos_logging::{attach, init};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[tokio::test(start_paused = true)]
async fn panic_message_reaches_log_key() {
    init(0);
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let key = log_key("fixture");
    let mut subscriber = backend.subscribe(&key).await.expect("subscribe");
    let publisher = attach(Arc::clone(&backend), key).await;
    let shutdown = CancellationToken::new();
    let tasks = TaskTracker::new();
    tasks.spawn(publisher.run(shutdown));

    let panic_result = std::panic::catch_unwind(|| {
        panic!("inventory mismatch");
    });
    assert!(panic_result.is_err());

    let sample = tokio::time::timeout(Duration::from_secs(1), subscriber.recv())
        .await
        .expect("panic log arrives")
        .expect("sample");
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert!(decoded.message.contains("inventory mismatch"));
}
