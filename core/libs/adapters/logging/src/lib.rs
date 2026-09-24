use thiserror::Error;
use tracing_subscriber::EnvFilter;

pub use log::{debug, error, info, trace, warn};

#[derive(Debug, Error)]
pub enum LogError {
    #[error("{0}")]
    Message(String),
}

pub fn init(verbose: u8) -> Result<(), LogError> {
    let default = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));
    let _ = tracing_log::LogTracer::init(); // already-initialized is success
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(|error| LogError::Message(error.to_string()))
}
