//! Spawns endpoint adapter tasks after startup commands and initial states.

use std::sync::Arc;

use tokio::sync::{mpsc, watch};

use blueos_api::{Message, cdr_encoding, jobs_key, service_liveliness_key};
use blueos_comms::Queryable;
use blueos_domain::Domain;
use blueos_idl::msg::blueos_msgs::{JobFeedbackList, JobList, ServiceMetrics};

use crate::{builder::AnswerQuery, service::ServiceError, tasks::hold_liveliness_until_cancelled};

use super::{
    super::{
        endpoints::{
            serve_command, serve_fixed_reply, serve_io_query, serve_query, serve_settings,
            serve_state, serve_update_settings,
        },
        types::{IntoInput, Kernel},
    },
    inbox::inbox_sender,
};

type PendingJobOutput = (
    Queryable,
    String,
    watch::Receiver<Option<bytes::Bytes>>,
    Queryable,
    String,
    watch::Receiver<Option<bytes::Bytes>>,
);

pub(crate) struct BootSpawnPlan<D: Domain> {
    pub info_queryable: Queryable,
    pub info_key: String,
    pub info_payload: bytes::Bytes,
    pub info_encoding: String,
    pub status_queryable: Queryable,
    pub status_key: String,
    pub status_encoding: String,
    pub status_latest: watch::Receiver<Option<bytes::Bytes>>,
    pub metrics_queryable: Queryable,
    pub metrics_key: String,
    pub pending_settings_serve: Option<(
        Queryable,
        String,
        String,
        watch::Receiver<Option<bytes::Bytes>>,
    )>,
    pub update_settings_queryable: Queryable,
    pub pending_commands: Vec<(Queryable, IntoInput<D>)>,
    pub jobs_queryable: Queryable,
    pub pending_job_outputs: Vec<PendingJobOutput>,
    pub pending_states: Vec<(
        Queryable,
        String,
        String,
        watch::Receiver<Option<bytes::Bytes>>,
    )>,
    pub pending_queries: Vec<(Queryable, AnswerQuery<D>)>,
    pub pending_io_queries: Vec<(Queryable, crate::builder::Respond, String)>,
}

struct InfoStatusMetricsSpawn {
    info_queryable: Queryable,
    info_key: String,
    info_payload: bytes::Bytes,
    info_encoding: String,
    status_queryable: Queryable,
    status_key: String,
    status_encoding: String,
    status_latest: watch::Receiver<Option<bytes::Bytes>>,
    metrics_queryable: Queryable,
    metrics_key: String,
}

pub(crate) async fn spawn_boot_endpoints<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    service: &'static str,
    plan: BootSpawnPlan<D>,
) -> Result<(), ServiceError> {
    let BootSpawnPlan {
        info_queryable,
        info_key,
        info_payload,
        info_encoding,
        status_queryable,
        status_key,
        status_encoding,
        status_latest,
        metrics_queryable,
        metrics_key,
        pending_settings_serve,
        update_settings_queryable,
        pending_commands,
        jobs_queryable,
        pending_job_outputs,
        pending_states,
        pending_queries,
        pending_io_queries,
    } = plan;

    spawn_info_status_and_metrics(
        kernel,
        InfoStatusMetricsSpawn {
            info_queryable,
            info_key,
            info_payload,
            info_encoding,
            status_queryable,
            status_key,
            status_encoding,
            status_latest,
            metrics_queryable,
            metrics_key,
        },
    );
    spawn_settings_and_commands(
        kernel,
        pending_settings_serve,
        update_settings_queryable,
        pending_commands,
    )?;
    spawn_job_and_state_endpoints(
        kernel,
        service,
        jobs_queryable,
        pending_job_outputs,
        pending_states,
    );
    spawn_query_endpoints(kernel, pending_queries, pending_io_queries);
    spawn_service_liveliness(kernel, service).await
}

fn spawn_info_status_and_metrics<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    plan: InfoStatusMetricsSpawn,
) {
    let InfoStatusMetricsSpawn {
        info_queryable,
        info_key,
        info_payload,
        info_encoding,
        status_queryable,
        status_key,
        status_encoding,
        status_latest,
        metrics_queryable,
        metrics_key,
    } = plan;
    kernel.endpoints.spawn(serve_fixed_reply(
        info_queryable,
        info_key,
        info_payload,
        info_encoding,
    ));
    kernel.endpoints.spawn(serve_state(
        status_queryable,
        status_key,
        status_encoding,
        status_latest,
    ));
    kernel.endpoints.spawn(serve_state(
        metrics_queryable,
        metrics_key,
        cdr_encoding(ServiceMetrics::SCHEMA_NAME),
        kernel.metrics_latest.subscribe(),
    ));
}

fn spawn_settings_and_commands<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    pending_settings_serve: Option<(
        Queryable,
        String,
        String,
        watch::Receiver<Option<bytes::Bytes>>,
    )>,
    update_settings_queryable: Queryable,
    pending_commands: Vec<(Queryable, IntoInput<D>)>,
) -> Result<(), ServiceError> {
    if let Some((queryable, key, encoding, latest)) = pending_settings_serve {
        kernel
            .endpoints
            .spawn(serve_settings(queryable, key, encoding, latest));
    }
    let inbox = mpsc::Sender::clone(inbox_sender(kernel)?);
    kernel.endpoints.spawn(serve_update_settings(
        update_settings_queryable,
        kernel
            .settings
            .as_ref()
            .map(|endpoint| Arc::clone(&endpoint.driver)),
        mpsc::Sender::clone(&inbox),
    ));
    for (queryable, into_input) in pending_commands {
        kernel.endpoints.spawn(serve_command(
            queryable,
            into_input,
            mpsc::Sender::clone(&inbox),
        ));
    }
    Ok(())
}

fn spawn_job_and_state_endpoints<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    service: &'static str,
    jobs_queryable: Queryable,
    pending_job_outputs: Vec<PendingJobOutput>,
    pending_states: Vec<(
        Queryable,
        String,
        String,
        watch::Receiver<Option<bytes::Bytes>>,
    )>,
) {
    kernel.endpoints.spawn(serve_state(
        jobs_queryable,
        jobs_key(service),
        cdr_encoding(JobList::SCHEMA_NAME),
        kernel.jobs_latest.subscribe(),
    ));
    for (feedback_queryable, feedback_key, feedback, history_queryable, history_key, history) in
        pending_job_outputs
    {
        kernel.endpoints.spawn(serve_state(
            feedback_queryable,
            feedback_key,
            cdr_encoding(JobFeedbackList::SCHEMA_NAME),
            feedback,
        ));
        kernel.endpoints.spawn(serve_state(
            history_queryable,
            history_key,
            cdr_encoding(JobList::SCHEMA_NAME),
            history,
        ));
    }
    for (queryable, key, encoding, latest) in pending_states {
        kernel
            .endpoints
            .spawn(serve_state(queryable, key, encoding, latest));
    }
}

fn spawn_query_endpoints<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    pending_queries: Vec<(Queryable, crate::builder::AnswerQuery<D>)>,
    pending_io_queries: Vec<(Queryable, crate::builder::Respond, String)>,
) {
    for (queryable, answer) in pending_queries {
        kernel.endpoints.spawn(serve_query::<D>(
            queryable,
            answer,
            Arc::clone(&kernel.snapshot_for_queries),
            Arc::clone(&kernel.clock),
        ));
    }
    for (queryable, respond, encoding) in pending_io_queries {
        kernel.endpoints.spawn(
            kernel
                .metrics
                .scope(serve_io_query(queryable, respond, encoding)),
        );
    }
}

async fn spawn_service_liveliness<D: Domain, Context: Send + Sync + 'static>(
    kernel: &Kernel<D, Context>,
    service: &'static str,
) -> Result<(), ServiceError> {
    let liveliness_key = service_liveliness_key(service);
    let liveliness = kernel
        .backend
        .declare_liveliness(&liveliness_key)
        .await
        .map_err(|source| ServiceError::DeclareEndpoint {
            key: liveliness_key,
            source,
        })?;
    let liveliness_shutdown = kernel.tasks.shutdown_token();
    let task_spawner = kernel.tasks.spawner();
    task_spawner.spawn(async move {
        hold_liveliness_until_cancelled(liveliness, liveliness_shutdown).await;
    });
    Ok(())
}
