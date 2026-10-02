//! Buffered records keep the timestamp from when they were created.

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime},
};

use blueos_api::log_key;
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_idl::{Message, msg::foxglove_msgs::Log};
use blueos_logging::{attach, init, set_time_source};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

static TIME_STEP: AtomicUsize = AtomicUsize::new(0);

#[tokio::test(start_paused = true)]
async fn replayed_records_keep_creation_timestamps() {
    TIME_STEP.store(0, Ordering::Relaxed);
    set_time_source(|| {
        let index = TIME_STEP.fetch_add(1, Ordering::Relaxed);
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000 + index as u64)
    });
    init(0);
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let key = log_key("fixture");
    let mut subscriber = backend.subscribe(&key).await.expect("subscribe");

    tracing::warn!(reason = "before session", "Settings file missing");

    let publisher = attach(Arc::clone(&backend), key).await;
    let shutdown = CancellationToken::new();
    let tasks = TaskTracker::new();
    tasks.spawn(publisher.run(shutdown));

    let sample = tokio::time::timeout(Duration::from_secs(1), subscriber.recv())
        .await
        .expect("log sample arrives")
        .expect("sample");
    assert_eq!(
        sample.timestamp(),
        Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000))
    );
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert_eq!(decoded.timestamp.sec, 1_700_000_000);
    assert_eq!(decoded.timestamp.nanosec, 0);
    assert!(decoded.message.contains("Settings file missing"));
}
