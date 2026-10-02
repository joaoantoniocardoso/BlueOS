//! A tracing event captured for console output and backbone publishing.

use core::fmt::Debug;
use std::time::SystemTime;

use tracing::Level;

/// One log line with the timestamp taken when the tracing event was handled.
#[derive(Clone, Debug)]
pub(crate) struct CapturedRecord {
    pub created_at: SystemTime,
    pub level: Level,
    pub message: String,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

struct WireMessageVisitor<'event> {
    message: &'event mut String,
    extras: &'event mut String,
}

impl tracing::field::Visit for WireMessageVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn Debug) {
        if field.name() == "message" {
            let formatted = format!("{value:?}");
            *self.message = trim_debug_quotes(&formatted);
            return;
        }
        let formatted = format!("{value:?}");
        let value_text = trim_debug_quotes(&formatted);
        self.extras
            .push_str(&format!(" {}={value_text}", field.name()));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            *self.message = value.to_string();
            return;
        }
        self.extras.push_str(&format!(" {}={value}", field.name()));
    }
}

/// Maps a [`Level`] to the Foxglove `Log` level constants (aligned with Python `commonwealth` producers).
pub(crate) fn foxglove_level(level: Level) -> u8 {
    use blueos_idl::msg::foxglove_msgs::constants_log::{DEBUG, ERROR, INFO, UNKNOWN, WARNING};

    match level {
        Level::TRACE => UNKNOWN,
        Level::DEBUG => DEBUG,
        Level::INFO => INFO,
        Level::WARN => WARNING,
        Level::ERROR => ERROR,
    }
}

/// Builds the Foxglove `Log.message` text: constant message first, then other fields like the console `fmt` layer.
pub(crate) fn wire_message_from_event(event: &tracing::Event<'_>) -> String {
    let mut message = String::new();
    let mut extras = String::new();
    let mut visitor = WireMessageVisitor {
        message: &mut message,
        extras: &mut extras,
    };
    event.record(&mut visitor);
    if message.is_empty() {
        message = event.metadata().name().to_string();
    }
    format!("{message}{extras}")
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
    use std::sync::{Arc, Mutex};

    use tracing_subscriber::{
        Layer,
        layer::{Context, SubscriberExt},
        registry::LookupSpan,
        util::SubscriberInitExt,
    };

    use super::{trim_debug_quotes, wire_message_from_event};

    struct CaptureLayer {
        message: Arc<Mutex<Option<String>>>,
    }

    impl<S> Layer<S> for CaptureLayer
    where
        S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
    {
        fn on_event(&self, event: &tracing::Event<'_>, _context: Context<'_, S>) {
            *self
                .message
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                Some(wire_message_from_event(event));
        }
    }

    #[test]
    fn trim_debug_quotes_strips_surrounding_quotes() {
        assert_eq!(trim_debug_quotes("\"hello\""), "hello");
        assert_eq!(trim_debug_quotes("hello"), "hello");
    }

    #[test]
    fn wire_message_includes_constant_message_and_structured_fields() {
        let message = Arc::new(Mutex::new(None));
        let _guard = tracing_subscriber::registry()
            .with(CaptureLayer {
                message: Arc::clone(&message),
            })
            .set_default();
        tracing::error!(
            service_error = "connect refused",
            "The service could not start or run"
        );
        assert_eq!(
            message
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_deref(),
            Some("The service could not start or run service_error=connect refused")
        );
    }
}
