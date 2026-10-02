//! Early failures are visible on the console and replay on the `log` key after attach.

use std::{sync::Arc, time::Duration};

use blueos_api::log_key;
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_idl::{Message, msg::foxglove_msgs::Log};
use blueos_logging::{attach, init_with_writer};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[tokio::test(start_paused = true)]
async fn failure_before_session_appears_on_console_and_log_key() {
    let console = Arc::new(std::sync::Mutex::new(Vec::new()));
    init_with_writer(0, Arc::clone(&console));
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let key = log_key("fixture");
    let mut subscriber = backend.subscribe(&key).await.expect("subscribe");

    tracing::error!(
        service_error = "connect refused",
        "The service could not start or run"
    );

    let publisher = attach(Arc::clone(&backend), key).await;
    let shutdown = CancellationToken::new();
    let tasks = TaskTracker::new();
    tasks.spawn(publisher.run(shutdown));

    let sample = tokio::time::timeout(Duration::from_secs(1), subscriber.recv())
        .await
        .expect("log sample arrives")
        .expect("sample");
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert!(
        decoded
            .message
            .contains("The service could not start or run")
    );
    assert!(decoded.message.contains("service_error=connect refused"));

    let console_bytes = console
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let console_text = String::from_utf8_lossy(&console_bytes);
    assert!(console_text.contains("connect refused"));
}
