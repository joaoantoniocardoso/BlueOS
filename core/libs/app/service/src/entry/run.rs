//! Start one Service from its command-line arguments.

use std::{ffi::OsString, process::ExitCode, sync::Arc};

use tracing::error;

use blueos_comms::CommsBackend;
use blueos_comms_zenoh::{ZenohBackend, config::ZenohConnectOptions};

use crate::{Kernel, RunOutcome, Service, ServiceContext, ServiceError, logging};

use super::{
    parse::{ParsedServiceArguments, parse_service_cli},
    system_clock::SystemClock,
    verbosity::verbosity_from_raw,
};

/// Starts logging, parses the CLI, opens the Session, runs the Kernel, and returns an exit code (D-29).
pub fn run<S: Service>(arguments: Vec<OsString>) -> ExitCode {
    logging::init_from_verbosity(verbosity_from_raw(&arguments));
    let parsed = match parse_service_cli::<S>(arguments) {
        Ok(parsed) => parsed,
        Err(error) => return error.exit_code(),
    };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("the tokio runtime builds");
    runtime.block_on(async {
        match run_with_log_publisher::<S>(parsed).await {
            Ok(RunOutcome::Stopped) => ExitCode::SUCCESS,
            Ok(RunOutcome::RepeatedInboxPanics) => ExitCode::from(1),
            Err(_service_error) => ExitCode::from(1),
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
    logging::init_from_verbosity(parsed.common.verbose);
    run_with_log_publisher_on_backend::<S>(parsed, backend, clock).await
}

async fn run_with_log_publisher<S: Service>(
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
    let clock = Arc::new(SystemClock::new());
    run_with_log_publisher_on_backend::<S>(parsed, backend, clock).await
}

async fn run_with_log_publisher_on_backend<S: Service>(
    parsed: ParsedServiceArguments<S::Arguments>,
    backend: Arc<dyn CommsBackend>,
    clock: Arc<dyn crate::Clock>,
) -> Result<RunOutcome, ServiceError> {
    let publisher = logging::attach_backbone(S::NAME, Arc::clone(&backend)).await;
    let log_runtime = logging::LogPublisherRuntime::start(publisher);
    let outcome = async {
        let context =
            ServiceContext::with_settings_path(parsed.service, parsed.common.settings_path);
        let builder = S::build(&context)?;
        let kernel = Kernel::start(S::NAME, builder, backend, clock).await?;
        Ok(kernel.run().await)
    }
    .await;
    if let Err(service_error) = &outcome {
        error!(%service_error, "The service could not start or run");
    }
    log_runtime.shutdown_and_wait().await;
    outcome
}
