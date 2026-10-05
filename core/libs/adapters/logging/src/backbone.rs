//! Publishes tracing records on the service `log` key through [`CommsBackend`].

use core::sync::atomic::{AtomicU64, Ordering};
use std::{
    collections::VecDeque,
    sync::{Arc, LazyLock, Mutex, Once, OnceLock},
    time::SystemTime,
};

use bytes::Bytes;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

use blueos_comms::{CommsBackend, Sample};
use blueos_idl::{
    Message,
    encoding::cdr_encoding,
    msg::{builtin_interfaces::Time, foxglove_msgs::Log},
};

use crate::record::{CapturedRecord, foxglove_level, wire_message_from_event};

// ponytail: 1024 pre-Session records (oldest dropped); live `try_send` Full drops increment
// DROPPED_LIVE without tracing (would recurse). Raise capacity if connect routinely logs more.
const PRE_SESSION_BUFFER_CAPACITY: usize = 1024;
const PUBLISH_CHANNEL_CAPACITY: usize = 1024;

static DROPPED_LIVE: AtomicU64 = AtomicU64::new(0);
static ENCODE_FAILURE: Once = Once::new();
static PUBLISH_FAILURE: Once = Once::new();
static TIME_SOURCE: OnceLock<fn() -> SystemTime> = OnceLock::new();
static STATE: LazyLock<Mutex<BackboneState>> = LazyLock::new(|| Mutex::new(BackboneState::new()));

struct BackboneState {
    pre_session: VecDeque<CapturedRecord>,
    live_sender: Option<mpsc::Sender<CapturedRecord>>,
}

/// Tracing layer that buffers until [`attach`] connects a [`CommsBackend`].
pub(crate) struct BackboneLayer;

/// Owns replay and live publishing on the service `log` key; run on a [`TaskTracker`], not detached.
pub struct LogPublisher {
    backend: Arc<dyn CommsBackend>,
    log_key: String,
    receiver: mpsc::Receiver<CapturedRecord>,
    replay: Vec<CapturedRecord>,
}

impl BackboneState {
    fn new() -> Self {
        Self {
            pre_session: VecDeque::new(),
            live_sender: None,
        }
    }

    fn push_pre_session(&mut self, record: CapturedRecord) {
        if self.pre_session.len() >= PRE_SESSION_BUFFER_CAPACITY {
            self.pre_session.pop_front();
        }
        self.pre_session.push_back(record);
    }
}

impl<S> Layer<S> for BackboneLayer
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _context: Context<'_, S>) {
        if should_ignore_target(event.metadata().target()) {
            return;
        }
        enqueue_captured_record(captured_record_from_event(event));
    }
}

impl LogPublisher {
    /// Replays buffered records, publishes live events, and drains the queue when `shutdown` fires.
    pub async fn run(mut self, shutdown: CancellationToken) {
        let encoding = cdr_encoding(Log::SCHEMA_NAME);
        for record in self.replay {
            publish_record(&self.backend, &self.log_key, &encoding, &record).await;
        }
        loop {
            tokio::select! {
                biased;
                _ = shutdown.cancelled() => {
                    drain_receiver(&mut self.receiver, &self.backend, &self.log_key, &encoding).await;
                    break;
                }
                record = self.receiver.recv() => {
                    match record {
                        Some(record) => {
                            publish_record(&self.backend, &self.log_key, &encoding, &record).await;
                        }
                        None => break,
                    }
                }
            }
        }
        drain_receiver(&mut self.receiver, &self.backend, &self.log_key, &encoding).await;
    }
}

fn now() -> SystemTime {
    TIME_SOURCE.get().copied().unwrap_or(SystemTime::now)()
}

/// Injectable clock for tests (`testing` feature).
#[cfg(feature = "testing")]
// qual:test_helper
pub fn set_time_source(source: fn() -> SystemTime) {
    let _ = TIME_SOURCE.set(source);
}

fn should_ignore_target(target: &str) -> bool {
    target.starts_with("zenoh")
}

fn captured_record_from_event(event: &tracing::Event<'_>) -> CapturedRecord {
    let metadata = event.metadata();
    CapturedRecord {
        created_at: now(),
        level: *metadata.level(),
        message: wire_message_from_event(event),
        target: metadata.target().to_string(),
        file: metadata.file().map(str::to_string),
        line: metadata.line(),
    }
}

fn enqueue_captured_record(record: CapturedRecord) {
    let mut state = STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(sender) = state.live_sender.as_ref() {
        match sender.try_send(record) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                DROPPED_LIVE.fetch_add(1, Ordering::Relaxed);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {}
        }
        return;
    }
    state.push_pre_session(record);
}

/// Connects the backbone layer to `backend` and returns a publisher the caller must run.
pub async fn attach(backend: Arc<dyn CommsBackend>, log_key: String) -> LogPublisher {
    let (sender, receiver) = mpsc::channel(PUBLISH_CHANNEL_CAPACITY);
    let replay = {
        let mut state = STATE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.live_sender = Some(sender);
        state.pre_session.drain(..).collect::<Vec<_>>()
    };
    LogPublisher {
        backend,
        log_key,
        receiver,
        replay,
    }
}

async fn drain_receiver(
    receiver: &mut mpsc::Receiver<CapturedRecord>,
    backend: &Arc<dyn CommsBackend>,
    log_key: &str,
    encoding: &str,
) {
    while let Ok(record) = receiver.try_recv() {
        publish_record(backend, log_key, encoding, &record).await;
    }
}

async fn publish_record(
    backend: &Arc<dyn CommsBackend>,
    log_key: &str,
    encoding: &str,
    record: &CapturedRecord,
) {
    let payload = match encode_log(record) {
        Ok(payload) => payload,
        Err(error) => {
            report_once(&ENCODE_FAILURE, || format!("CDR encode failed: {error}"));
            return;
        }
    };
    let sample =
        Sample::new(log_key, Bytes::from(payload), encoding).with_timestamp(record.created_at);
    if let Err(error) = backend.publish(sample).await {
        report_once(&PUBLISH_FAILURE, || {
            format!("backbone publish failed: {error}")
        });
    }
}

fn report_once(report: &Once, detail: impl FnOnce() -> String) {
    report.call_once(|| {
        // Tracing would recurse through BackboneLayer; stderr once keeps failures visible.
        eprintln!("blueos-logging: {}", detail());
    });
}

fn encode_log(record: &CapturedRecord) -> Result<Vec<u8>, blueos_idl::error::Error> {
    let duration = record
        .created_at
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let timestamp = Time {
        sec: duration.as_secs().try_into().unwrap_or(i32::MAX),
        nanosec: duration.subsec_nanos(),
    };
    let file = record.file.clone().unwrap_or_default();
    let message = Log {
        timestamp,
        level: foxglove_level(record.level),
        message: record.message.clone(),
        name: record.target.clone(),
        file,
        line: record.line.unwrap_or_default(),
    };
    message.encode()
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::SystemTime,
    };

    use tracing::Level;
    use tracing_subscriber::{
        Layer,
        layer::{Context, SubscriberExt},
        registry::LookupSpan,
        util::SubscriberInitExt,
    };

    use super::{
        BackboneState, CapturedRecord, captured_record_from_event, enqueue_captured_record,
        should_ignore_target,
    };

    struct CaptureLayer {
        record: Arc<Mutex<Option<CapturedRecord>>>,
    }

    impl<S> Layer<S> for CaptureLayer
    where
        S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
    {
        fn on_event(&self, event: &tracing::Event<'_>, _context: Context<'_, S>) {
            *self
                .record
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                Some(captured_record_from_event(event));
        }
    }

    #[test]
    fn captured_record_from_event_builds_message_and_target() {
        let record = Arc::new(Mutex::new(None));
        let _guard = tracing_subscriber::registry()
            .with(CaptureLayer {
                record: Arc::clone(&record),
            })
            .set_default();
        tracing::warn!(target: "blueos_test", "backbone capture");
        let guard = record
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let captured = guard.as_ref().expect("record");
        assert_eq!(captured.target, "blueos_test");
        assert_eq!(captured.message, "backbone capture");
    }

    #[test]
    fn zenoh_targets_are_ignored() {
        assert!(should_ignore_target("zenoh::api::session"));
        assert!(!should_ignore_target("blueos_comms_zenoh"));
    }

    #[test]
    fn enqueue_uses_pre_session_when_live_sender_absent() {
        let record = CapturedRecord {
            created_at: SystemTime::UNIX_EPOCH,
            level: Level::INFO,
            message: "queued".into(),
            target: "test".into(),
            file: None,
            line: None,
        };
        enqueue_captured_record(record);
    }

    #[test]
    fn pre_session_buffer_drops_oldest_at_capacity() {
        let mut state = BackboneState::new();
        for index in 0..=super::PRE_SESSION_BUFFER_CAPACITY {
            state.push_pre_session(CapturedRecord {
                created_at: SystemTime::UNIX_EPOCH,
                level: Level::INFO,
                message: index.to_string(),
                target: "test".into(),
                file: None,
                line: None,
            });
        }
        assert_eq!(state.pre_session.len(), super::PRE_SESSION_BUFFER_CAPACITY);
        assert_eq!(state.pre_session.front().unwrap().message, "1");
        assert_eq!(
            state.pre_session.back().unwrap().message,
            super::PRE_SESSION_BUFFER_CAPACITY.to_string()
        );
    }
}
