//! Start one Service from its command-line arguments.

use std::{ffi::OsString, process::ExitCode, sync::Arc};

use tracing::error;
use tracing_subscriber::EnvFilter;

use blueos_comms::CommsBackend;
use blueos_comms_zenoh::{ZenohBackend, config::ZenohConnectOptions};

use crate::{Kernel, RunOutcome, Service, ServiceContext, ServiceError};

use super::{
    parse::{ParsedServiceArguments, parse_service_cli},
    system_clock::SystemClock,
    verbosity::verbosity_from_raw,
};

/// Starts logging, parses the CLI, opens the Session, runs the Kernel, and returns an exit code (D-29).
pub fn run<S: Service>(arguments: Vec<OsString>) -> ExitCode {
    init_logging(verbosity_from_raw(&arguments));
    let parsed = match parse_service_cli::<S>(arguments) {
        Ok(parsed) => parsed,
        Err(error) => return error.exit_code(),
    };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("the tokio runtime builds");
    runtime.block_on(async {
        match run_parsed::<S>(parsed).await {
            Ok(RunOutcome::Stopped) => ExitCode::SUCCESS,
            Err(service_error) => {
                error!(%service_error, "The service could not start or run");
                ExitCode::from(1)
            }
        }
    })
}

/// Runs a parsed CLI on an injected backbone (layer L3, channel backend in tests).
#[cfg(feature = "testing")]
pub async fn run_with_backend<S: Service>(
    parsed: ParsedServiceArguments<S::Arguments>,
    backend: Arc<dyn CommsBackend>,
    clock: Arc<dyn crate::Clock>,
) -> Result<RunOutcome, ServiceError> {
    let context = ServiceContext::with_settings_path(parsed.service, parsed.common.settings_path);
    let builder = S::build(&context)?;
    let kernel = Kernel::start(S::NAME, builder, backend, clock).await?;
    Ok(kernel.run().await)
}

async fn run_parsed<S: Service>(
    parsed: ParsedServiceArguments<S::Arguments>,
) -> Result<RunOutcome, ServiceError> {
    let options = ZenohConnectOptions {
        endpoint: &parsed.common.zenoh_endpoint,
        config_file: parsed.common.zenoh_config.as_deref(),
        zenoh_sets: &parsed.common.zenoh_sets,
    };
    let backend: Arc<dyn CommsBackend> = Arc::new(
        ZenohBackend::connect_with_options(&options)
            .await
            .map_err(ServiceError::Session)?,
    );
    let context = ServiceContext::with_settings_path(parsed.service, parsed.common.settings_path);
    let builder = S::build(&context)?;
    let clock = Arc::new(SystemClock::new());
    let kernel = Kernel::start(S::NAME, builder, backend, clock).await?;
    Ok(kernel.run().await)
}

fn init_logging(verbosity: u8) {
    let default_level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
