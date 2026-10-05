//! Harness startup: build context and spawn the Kernel task.

use std::sync::Arc;

use tokio::task::JoinSet;

use crate::{Kernel, Service, ServiceContext, ServiceError, metrics_registry::MetricsRegistry};

use super::{Harness, PausedClock};

type BuiltServiceContext<S> = (
    <S as Service>::Context,
    crate::builder::ServiceBuilder<<S as Service>::Domain, <S as Service>::Context>,
);

pub(super) async fn start_on_with_effect_log<S: Service>(
    backend: Arc<dyn blueos_comms::CommsBackend>,
    mut service: ServiceContext<S::Arguments>,
    change: impl FnOnce(&mut S::Context),
    effect_log: Option<crate::kernel::EffectLogStorage<S::Domain>>,
) -> Result<Harness<S>, ServiceError> {
    let settings_directory = ensure_settings_directory::<S>(&mut service)?;
    let metrics = MetricsRegistry::default();
    let (context, mut builder) = build_service_context::<S>(&service, &metrics, change)?;
    builder.metrics = metrics;
    let shutdown = builder.shutdown_handle();
    let clock = Arc::new(PausedClock::start());
    let kernel = Kernel::start_with_effect_log(
        S::NAME,
        builder,
        context,
        Arc::clone(&backend),
        clock,
        effect_log,
    )
    .await?;
    spawn_harness(backend, service, settings_directory, shutdown, kernel).await
}

fn ensure_settings_directory<S: Service>(
    service: &mut ServiceContext<S::Arguments>,
) -> Result<Option<tempfile::TempDir>, ServiceError> {
    if service.settings_path.is_some() {
        return Ok(None);
    }
    let directory = tempfile::tempdir().map_err(|error| ServiceError::Build(error.into()))?;
    service.settings_path = Some(directory.path().to_path_buf());
    Ok(Some(directory))
}

fn build_service_context<S: Service>(
    service: &ServiceContext<S::Arguments>,
    metrics: &MetricsRegistry,
    change: impl FnOnce(&mut S::Context),
) -> Result<BuiltServiceContext<S>, ServiceError> {
    metrics::with_local_recorder(metrics, || {
        let mut context = S::context(service)?;
        change(&mut context);
        let builder = S::build(service, &context)?.for_service::<S>(service);
        Ok((context, builder))
    })
}

async fn spawn_harness<S: Service>(
    backend: Arc<dyn blueos_comms::CommsBackend>,
    _service: ServiceContext<S::Arguments>,
    settings_directory: Option<tempfile::TempDir>,
    shutdown: crate::ShutdownHandle,
    kernel: Kernel<S::Domain, S::Context>,
) -> Result<Harness<S>, ServiceError> {
    let command_sender = kernel.command_sender().ok_or_else(|| {
        ServiceError::Build("the Kernel did not hand out a CommandSender before it ran".into())
    })?;
    let durable_flush = kernel.durable_write_flush();
    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        kernel.run().await;
    });
    Ok(Harness {
        backend,
        command_sender,
        durable_flush,
        shutdown,
        kernel: tasks,
        service: core::marker::PhantomData,
        _settings_directory: settings_directory,
    })
}
