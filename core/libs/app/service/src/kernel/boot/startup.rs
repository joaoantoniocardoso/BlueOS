//! Startup Commands and initial publication before endpoint adapters run.

use std::sync::Arc;

use tokio::sync::mpsc;

use blueos_domain::Domain;

use crate::{
    builder::InboxCommand,
    command_sender::CommandSender,
    inbox::{Delivery, Input},
    service::ServiceError,
};

use super::{
    super::{endpoints::publish_standard_status, jobs::encode_job_outputs, types::Kernel},
    inbox::inbox_sender,
};

pub(crate) async fn run_startup_commands<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    startup_commands: Vec<InboxCommand<D>>,
) -> Result<(), ServiceError> {
    for command in startup_commands {
        if kernel
            .dispatch(Delivery {
                input: Input::Command(command),
                reply: None,
                persist_settings: false,
            })
            .await
            .is_some()
        {
            return Err(ServiceError::Build(
                "the Kernel stopped during startup after repeated Inbox panics".into(),
            ));
        }
    }
    Ok(())
}

pub(crate) async fn publish_initial_states<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    status_key: &str,
    status_encoding: &str,
    status_latest: &tokio::sync::watch::Sender<Option<bytes::Bytes>>,
) {
    let initial_states = kernel
        .states
        .iter()
        .map(|state| (state.endpoint.project)(&kernel.snapshot))
        .collect();
    kernel.publish_states(initial_states).await;
    kernel.publish_jobs().await;
    kernel
        .publish_job_feedback(encode_job_outputs(
            &kernel.job_outputs,
            &kernel.snapshot,
            kernel.jobs(),
            kernel.jobs(),
        ))
        .await;
    kernel.publish_settings().await;
    kernel.projections.refresh(&kernel.snapshot);
    publish_standard_status(&kernel.backend, status_key, status_encoding, status_latest).await;
}

pub(crate) fn start_supervised_tasks<D: Domain, Context: Send + Sync + 'static>(
    kernel: &Kernel<D, Context>,
    task_specs: Vec<crate::tasks::TaskSpec<D, Context>>,
    session: crate::command_sender::Session,
) -> Result<(), ServiceError> {
    let command_sender = CommandSender::new(mpsc::Sender::clone(inbox_sender(kernel)?));
    kernel.tasks.start(
        task_specs,
        session,
        command_sender,
        Arc::clone(&kernel.context),
        Arc::clone(&kernel.clock),
    );
    Ok(())
}
