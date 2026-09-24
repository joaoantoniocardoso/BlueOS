use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use blueos_comms::{Payload, Session};
use tokio::sync::mpsc;
use tracing::Level;
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

const LOG_CHANNEL_CAPACITY: usize = 1024;

type Encoder = Arc<dyn Fn(&LogRecord) -> (Payload, String) + Send + Sync>;

static ENCODER: OnceLock<Encoder> = OnceLock::new();
static PUBLISH_SENDER: OnceLock<mpsc::Sender<PublishMessage>> = OnceLock::new();

struct PublishMessage {
    payload: Payload,
    encoding: String,
}

#[derive(Clone, Debug)]
pub struct LogRecord {
    pub level: Level,
    pub message: String,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

pub fn log_key_for_service(service_name: &str) -> String {
    format!("blueos/v1/{service_name}/log")
}

pub struct ZenohLogLayer {
    dropped: Arc<AtomicU64>,
}

impl ZenohLogLayer {
    pub fn new() -> Self {
        Self {
            dropped: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl<S> Layer<S> for ZenohLogLayer
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _context: Context<'_, S>) {
        let encoder = match ENCODER.get() {
            Some(encoder) => encoder,
            None => return,
        };
        let sender = match PUBLISH_SENDER.get() {
            Some(sender) => sender,
            None => return,
        };
        let record = event_to_record(event);
        let (payload, encoding) = encoder(&record);
        match sender.try_send(PublishMessage { payload, encoding }) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {}
        }
    }
}

pub struct ZenohLogGuard {
    _task: tokio::task::JoinHandle<()>,
}

pub async fn attach_zenoh_publisher<EncoderFn>(
    session: Session,
    log_key: String,
    encoder: EncoderFn,
) -> Result<ZenohLogGuard, String>
where
    EncoderFn: Fn(&LogRecord) -> (Payload, String) + Send + Sync + 'static,
{
    if PUBLISH_SENDER.get().is_some() {
        return Err("zenoh log publisher already attached".into());
    }
    let _ = ENCODER.set(Arc::new(encoder));
    let (sender, mut receiver) = mpsc::channel(LOG_CHANNEL_CAPACITY);
    let _ = PUBLISH_SENDER.set(sender);
    let task = tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            let _ = session
                .publish(&log_key, message.payload, &message.encoding, None)
                .await;
        }
    });
    Ok(ZenohLogGuard { _task: task })
}

fn event_to_record(event: &tracing::Event<'_>) -> LogRecord {
    let mut message = String::new();
    let mut visitor = MessageVisitor {
        message: &mut message,
    };
    event.record(&mut visitor);
    if message.is_empty() {
        message = event.metadata().name().to_string();
    }
    let metadata = event.metadata();
    LogRecord {
        level: *metadata.level(),
        message,
        target: metadata.target().to_string(),
        file: metadata.file().map(|value| value.to_string()),
        line: metadata.line(),
    }
}

struct MessageVisitor<'event> {
    message: &'event mut String,
}

impl tracing::field::Visit for MessageVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let formatted = format!("{value:?}");
            *self.message = trim_debug_quotes(&formatted);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            *self.message = value.to_string();
        }
    }
}

fn trim_debug_quotes(value: &str) -> String {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::trim_debug_quotes;

    #[test]
    fn trim_debug_quotes_strips_surrounding_quotes() {
        assert_eq!(trim_debug_quotes("\"hello\""), "hello");
        assert_eq!(trim_debug_quotes("hello"), "hello");
    }
}
