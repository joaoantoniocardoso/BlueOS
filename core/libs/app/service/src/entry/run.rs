//! Start one Service from its command-line arguments.

use std::{ffi::OsString, process::ExitCode, sync::Arc};

use tracing::error;

use blueos_comms::CommsBackend;
use blueos_comms_zenoh::{ZenohBackend, config::ZenohConnectOptions};

use crate::{
    Kernel, RunOutcome, Service, ServiceContext, ServiceError, logging,
    metrics_registry::MetricsRegistry,
};

use super::{
    parse::{ParsedServiceArguments, parse_service_cli},
    system_clock::SystemClock,
    verbosity::verbosity_from_raw,
};

/// Starts logging, parses the CLI, opens the Session, calls `context` and then `build`, runs the Kernel, and returns
/// an exit code (D-29).
pub fn run<S: Service>(arguments: Vec<OsString>) -> ExitCode {
    logging::init_from_verbosity(verbosity_from_raw(&arguments));
    let parsed = match parse_service_cli::<S>(arguments) {
        Ok(parsed) => parsed,
        Err(error) => return error.exit_code(),
    };
    run_parsed_service::<S>(parsed)
}

fn run_parsed_service<S: Service>(parsed: ParsedServiceArguments<S::Arguments>) -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            error!(%error, "The tokio runtime could not start");
            return ExitCode::from(1);
        }
    };
    runtime.block_on(async { exit_code_for_outcome(run_with_log_publisher::<S>(parsed).await) })
}

fn exit_code_for_outcome(outcome: Result<RunOutcome, ServiceError>) -> ExitCode {
    match outcome {
        Ok(RunOutcome::Stopped) => ExitCode::SUCCESS,
        Ok(RunOutcome::RepeatedInboxPanics) => ExitCode::from(1),
        Err(_service_error) => ExitCode::from(1),
    }
}

/// Runs a parsed CLI on an injected backbone (layer L3, channel backend in tests). The Service's metrics are not
/// installed as the process-wide recorder, so only what its own tasks record reaches them.
// qual:test_helper
#[cfg(feature = "testing")]
pub async fn run_with_backend<S: Service>(
    parsed: ParsedServiceArguments<S::Arguments>,
    backend: Arc<dyn CommsBackend>,
    clock: Arc<dyn crate::Clock>,
) -> Result<RunOutcome, ServiceError> {
    logging::init_from_verbosity(parsed.common.verbose);
    run_with_log_publisher_on_backend::<S>(parsed, backend, clock, MetricsRegistry::default()).await
}

async fn run_with_log_publisher<S: Service>(
    parsed: ParsedServiceArguments<S::Arguments>,
) -> Result<RunOutcome, ServiceError> {
    let metrics = MetricsRegistry::default();
    metrics.install();
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
    run_with_log_publisher_on_backend::<S>(parsed, backend, clock, metrics).await
}

async fn run_with_log_publisher_on_backend<S: Service>(
    parsed: ParsedServiceArguments<S::Arguments>,
    backend: Arc<dyn CommsBackend>,
    clock: Arc<dyn crate::Clock>,
    metrics: MetricsRegistry,
) -> Result<RunOutcome, ServiceError> {
    let publisher = logging::attach_backbone(S::NAME, Arc::clone(&backend)).await;
    let log_runtime = logging::LogPublisherRuntime::start(publisher);
    let started = async {
        let service = ServiceContext::with_settings_path(
            parsed.service,
            parsed.common.settings_path,
            Arc::clone(&backend),
        );
        let context = S::context(&service)?;
        let mut builder = S::build(&service, &context)?.for_service::<S>(&service);
        builder.metrics = metrics;
        Kernel::start(S::NAME, builder, context, backend, clock).await
    }
    .await;
    let mut kernel = match started {
        Ok(kernel) => kernel,
        Err(service_error) => {
            error!(%service_error, "The service could not start or run");
            log_runtime.shutdown_and_wait().await;
            return Err(service_error);
        }
    };
    kernel.attach_log_publisher(log_runtime);
    Ok(kernel.run().await)
}
