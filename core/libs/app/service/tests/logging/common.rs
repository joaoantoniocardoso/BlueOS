//! Shared setup for logging integration tests.

#![expect(
    dead_code,
    reason = "each logging integration test uses a subset of the shared helpers"
)]

use core::time::Duration;
use std::sync::Arc;

use tokio_util::{sync::CancellationToken, task::TaskTracker};

use blueos_api::log_key;
use blueos_comms::{CommsBackend, Subscriber, channel::ChannelBackend};
use blueos_logging::{attach, init};

pub(crate) struct LogPublisherFixture {
    pub(crate) backend: Arc<dyn CommsBackend>,
    pub(crate) subscriber: Subscriber,
    pub(crate) shutdown: CancellationToken,
    pub(crate) tasks: TaskTracker,
}

pub(crate) async fn start_log_publisher(service_name: &str, log_level: u8) -> LogPublisherFixture {
    init(log_level);
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let key = log_key(service_name);
    let subscriber = backend.subscribe(&key).await.expect("subscribe");
    let publisher = attach(Arc::clone(&backend), key).await;
    let shutdown = CancellationToken::new();
    let tasks = TaskTracker::new();
    tasks.spawn(publisher.run(shutdown.clone()));
    LogPublisherFixture {
        backend,
        subscriber,
        shutdown,
        tasks,
    }
}

pub(crate) async fn recv_log_sample(
    subscriber: &mut Subscriber,
    timeout_message: &str,
) -> blueos_comms::Sample {
    tokio::time::timeout(Duration::from_secs(1), subscriber.recv())
        .await
        .expect(timeout_message)
        .expect("sample")
}
