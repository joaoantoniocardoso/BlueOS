//! Kernel `boot` orchestration.

mod declare;
mod declare_core_endpoints;
mod declare_durable;
mod declare_jobs;
mod declare_queries;
mod declare_settings;

use bytes::Bytes;
use tokio::sync::watch;

use blueos_domain::Domain;

use crate::{
    builder::InboxCommand, command_sender::Session, service::ServiceError, tasks::TaskSpec,
};

use super::super::types::Kernel;
use super::request::KernelBootRequest;
use super::spawn::{BootSpawnPlan, spawn_boot_endpoints};
use super::startup::{publish_initial_states, run_startup_commands, start_supervised_tasks};

pub(super) struct BootDeclared<D: Domain, Context> {
    pub kernel: Kernel<D, Context>,
    pub startup_commands: Vec<InboxCommand<D>>,
    pub task_specs: Vec<TaskSpec<D, Context>>,
    pub session: Session,
    pub service: &'static str,
    pub status_key: String,
    pub status_encoding: String,
    pub status_latest: watch::Sender<Option<Bytes>>,
    pub spawn_plan: BootSpawnPlan<D>,
}

pub(crate) async fn boot<D: Domain, Context: Send + Sync + 'static>(
    request: KernelBootRequest<D, Context>,
) -> Result<Kernel<D, Context>, ServiceError> {
    let declared = Kernel::<D, Context>::declare_boot(request).await?;
    finish_boot(declared).await
}

async fn finish_boot<D: Domain, Context: Send + Sync + 'static>(
    declared: BootDeclared<D, Context>,
) -> Result<Kernel<D, Context>, ServiceError> {
    let BootDeclared {
        mut kernel,
        startup_commands,
        task_specs,
        session,
        service,
        status_key,
        status_encoding,
        status_latest,
        spawn_plan,
    } = declared;
    run_startup_commands(&mut kernel, startup_commands).await?;
    publish_initial_states(&mut kernel, &status_key, &status_encoding, &status_latest).await;
    start_supervised_tasks(&kernel, task_specs, session)?;
    kernel.publish_metrics().await;
    spawn_boot_endpoints(&mut kernel, service, spawn_plan).await?;
    Ok(kernel)
}
