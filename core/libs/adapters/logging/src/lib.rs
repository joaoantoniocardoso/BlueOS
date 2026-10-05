//! Prints a service's logs to the console and publishes them on the backbone as Foxglove `Log` messages.
//!
//! Call [`init`] once per process on the first line of service entry. Attach [`attach`] when a
//! [`CommsBackend`] exists so buffered records replay on the service `log` key.

#![expect(
    clippy::pub_use,
    reason = "blueos-logging re-exports attach and LogPublisher for service entry"
)]

mod backbone;
mod record;

use std::sync::Once;
#[cfg(feature = "testing")]
use std::sync::{Arc, Mutex};

use tracing_subscriber::{
    EnvFilter, fmt::MakeWriter, layer::SubscriberExt, util::SubscriberInitExt,
};

#[cfg(feature = "testing")]
pub use backbone::set_time_source;
pub use backbone::{LogPublisher, attach};

static INIT: Once = Once::new();

#[cfg(feature = "testing")]
struct ConsoleWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

#[cfg(feature = "testing")]
impl std::io::Write for ConsoleWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        let mut buffer = self
            .buffer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        buffer.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Starts console logging and installs the backbone layer and panic hook.
pub fn init(verbosity: u8) {
    INIT.call_once(|| {
        install_panic_hook();
        install_subscriber(verbosity, std::io::stderr);
    });
}

/// Starts logging with a custom writer (tests only).
#[cfg(feature = "testing")]
// qual:test_helper
pub fn init_with_writer(verbosity: u8, writer: Arc<Mutex<Vec<u8>>>) {
    INIT.call_once(|| {
        install_panic_hook();
        let console = Arc::clone(&writer);
        install_subscriber(verbosity, move || ConsoleWriter {
            buffer: Arc::clone(&console),
        });
    });
}

fn install_subscriber(
    verbosity: u8,
    writer: impl for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
) {
    let default_level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(writer))
        .with(backbone::BackboneLayer)
        .try_init();
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let message = panic_message(panic_info.payload());
        let location = panic_location(panic_info.location());
        tracing::error!(%message, location = %location, "panic");
    }));
}

// qual:allow(coupling, deh) reason: "panic hook reads the payload the runtime provides"
fn panic_message(payload: &(dyn core::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        text.to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "panic".to_string()
    }
}

fn panic_location(location: Option<&core::panic::Location<'_>>) -> String {
    location
        .map(|location| {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        })
        .unwrap_or_default()
}
