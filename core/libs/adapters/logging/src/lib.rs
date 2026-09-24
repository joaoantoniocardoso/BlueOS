//! Tracing subscriber with optional Zenoh log publishing (D-13).
//!
//! Call [`init`] once per process. Attach [`ZenohLogGuard`] when the comms session is ready;
//! records are encoded by your closure (CDR foxglove `Log` once `blueos-idl` is wired).

mod zenoh_layer;

use std::sync::Once;

use thiserror::Error;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub use tracing::{debug, error, info, trace, warn};

pub use zenoh_layer::{LogRecord, ZenohLogGuard, attach_zenoh_publisher, log_key_for_service};

#[derive(Debug, Error)]
pub enum LogError {
    #[error("{0}")]
    Message(String),
}

pub fn init(service_name: &str, verbosity: u8) -> Result<(), LogError> {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let default = match verbosity {
            0 => "info",
            1 => "debug",
            _ => "trace",
        };
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));
        let _ = tracing_log::LogTracer::init();
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().with_target(true))
            .with(zenoh_layer::ZenohLogLayer::new())
            .try_init();
    });
    info!("Starting {service_name}");
    Ok(())
}
