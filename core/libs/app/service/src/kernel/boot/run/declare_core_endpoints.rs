//! Inbox channel, service manifest, and info/status/metrics queryables.

use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::{mpsc, watch};

use blueos_api::{
    Message, cdr_encoding, command_key, info_query_key, job_feedback_key, job_history_key,
    job_result_key, jobs_key, state_key, status_state_key,
};
use blueos_comms::{CommsBackend, Queryable};
use blueos_domain::Domain;
use blueos_idl::msg::blueos_msgs::{
    EndpointInfo, JobFeedbackList, JobList, JobResult, ServiceInfo, ServiceMetrics, ServiceStatus,
};

use crate::{
    builder::ServiceBuilder, inbox::Delivery, runtime_gauges::RuntimeGauges, service::ServiceError,
    tasks::TaskSupervisor,
};

use super::super::super::{
    endpoints::{carrying, declare},
    types::{INBOX_CAPACITY, Kernel, METRICS, UPDATE_SETTINGS, UPDATE_SETTINGS_ACTION},
};

pub(super) struct InboxBootPrepared<D: Domain> {
    pub inbox: mpsc::Receiver<Delivery<D>>,
    pub inbox_sender: mpsc::Sender<Delivery<D>>,
    pub snapshot_for_queries: Arc<tokio::sync::RwLock<D::Snapshot>>,
    pub job_type_names: Vec<String>,
    pub service_info: ServiceInfo,
}

pub(super) struct CoreQueryablesBootPrepared {
    pub metrics_key: String,
    pub info_key: String,
    pub info_encoding: String,
    pub info_payload: Bytes,
    pub info_queryable: Queryable,
    pub status_key: String,
    pub status_encoding: String,
    pub status_latest: watch::Sender<Option<Bytes>>,
    pub status_queryable: Queryable,
    pub metrics_queryable: Queryable,
    pub inbox_step_time: metrics::Histogram,
    pub inbox_depth: metrics::Gauge,
    pub runtime_gauges: RuntimeGauges,
    pub task_supervisor: TaskSupervisor,
    pub update_settings_queryable: Queryable,
}

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    pub(super) fn prepare_inbox_and_service_info(
        service: &'static str,
        builder: &ServiceBuilder<D, Context>,
    ) -> InboxBootPrepared<D> {
        let (inbox_sender, inbox) = mpsc::channel(INBOX_CAPACITY);
        let snapshot_for_queries = Arc::new(tokio::sync::RwLock::new(builder.snapshot.clone()));
        let job_type_names = collect_boot_job_type_names(builder);
        let service_info = build_boot_service_info(service, builder, &job_type_names);
        InboxBootPrepared {
            inbox,
            inbox_sender,
            snapshot_for_queries,
            job_type_names,
            service_info,
        }
    }

    pub(super) async fn declare_core_service_queryables(
        service: &'static str,
        backend: Arc<dyn CommsBackend>,
        builder: &ServiceBuilder<D, Context>,
        inbox_prepared: &InboxBootPrepared<D>,
    ) -> Result<CoreQueryablesBootPrepared, ServiceError> {
        let metrics_key = state_key(service, METRICS);
        let info_key = info_query_key(service);
        let info_encoding = cdr_encoding(ServiceInfo::SCHEMA_NAME);
        let info_payload = Bytes::from(
            inbox_prepared
                .service_info
                .encode()
                .map_err(|error| ServiceError::Build(Box::new(error)))?,
        );
        let info_queryable = declare(&*backend, info_key.clone()).await?;
        let status_key = status_state_key(service);
        let status_encoding = cdr_encoding(ServiceStatus::SCHEMA_NAME);
        let status_latest = watch::Sender::new(None);
        let status_queryable = declare(&*backend, status_key.clone()).await?;
        let metrics_queryable = declare(&*backend, metrics_key.clone()).await?;
        let (inbox_step_time, inbox_depth) = metrics::with_local_recorder(&builder.metrics, || {
            (
                metrics::histogram!("inbox_step_seconds"),
                metrics::gauge!("inbox_depth"),
            )
        });
        let runtime_gauges = RuntimeGauges::new(&builder.metrics);
        let task_supervisor = TaskSupervisor::new(
            service,
            Arc::clone(&backend),
            status_key.clone(),
            status_latest.clone(),
            builder.metrics.clone(),
        );
        let update_settings_queryable =
            declare(&*backend, command_key(service, UPDATE_SETTINGS)).await?;
        Ok(CoreQueryablesBootPrepared {
            metrics_key,
            info_key,
            info_encoding,
            info_payload,
            info_queryable,
            status_key,
            status_encoding,
            status_latest,
            status_queryable,
            metrics_queryable,
            inbox_step_time,
            inbox_depth,
            runtime_gauges,
            task_supervisor,
            update_settings_queryable,
        })
    }
}

fn collect_boot_job_type_names<D: Domain, Context>(
    builder: &ServiceBuilder<D, Context>,
) -> Vec<String> {
    builder
        .commands
        .iter()
        .map(|command| command.name.clone())
        .chain([UPDATE_SETTINGS.to_owned()])
        .collect()
}

fn build_boot_service_info<D: Domain, Context>(
    service: &'static str,
    builder: &ServiceBuilder<D, Context>,
    job_type_names: &[String],
) -> ServiceInfo {
    ServiceInfo {
        name: service.to_owned(),
        version: builder.metadata.version.to_owned(),
        build: builder.metadata.build.to_owned(),
        capabilities: builder
            .metadata
            .capabilities
            .iter()
            .map(|capability| (*capability).to_owned())
            .collect(),
        endpoints: builder
            .manifest_endpoints
            .iter()
            .cloned()
            .chain(standard_manifest_endpoints(service))
            .chain(job_type_manifest_endpoints(
                service,
                builder,
                job_type_names,
            ))
            .collect(),
    }
}

fn manifest_endpoint(
    kind: &str,
    name: String,
    key: String,
    interface_type: String,
    schema: String,
) -> EndpointInfo {
    EndpointInfo {
        kind: kind.to_owned(),
        name,
        key,
        interface_type,
        schema,
    }
}

fn standard_manifest_endpoints(service: &'static str) -> [EndpointInfo; 3] {
    let metrics_key = state_key(service, METRICS);
    [
        manifest_endpoint(
            "job",
            UPDATE_SETTINGS.to_owned(),
            command_key(service, UPDATE_SETTINGS),
            UPDATE_SETTINGS_ACTION.to_owned(),
            blueos_idl::schema(UPDATE_SETTINGS_ACTION)
                .unwrap_or_default()
                .to_owned(),
        ),
        manifest_endpoint(
            "state",
            "jobs".to_owned(),
            jobs_key(service),
            JobList::SCHEMA_NAME.to_owned(),
            JobList::SCHEMA.to_owned(),
        ),
        manifest_endpoint(
            "state",
            METRICS.to_owned(),
            metrics_key,
            ServiceMetrics::SCHEMA_NAME.to_owned(),
            ServiceMetrics::SCHEMA.to_owned(),
        ),
    ]
}

fn job_type_manifest_endpoints<'a, D: Domain, Context>(
    service: &'static str,
    builder: &'a ServiceBuilder<D, Context>,
    job_type_names: &'a [String],
) -> impl Iterator<Item = EndpointInfo> + 'a {
    job_type_names.iter().flat_map(|job_type| {
        let (feedback_type, result_type) = builder
            .job_outputs
            .get(job_type)
            .map(|output| (output.feedback_type, output.result_type))
            .unwrap_or_default();
        [
            manifest_endpoint(
                "state",
                format!("jobs/{job_type}/feedback"),
                job_feedback_key(service, job_type),
                JobFeedbackList::SCHEMA_NAME.to_owned(),
                carrying::<JobFeedbackList>(feedback_type),
            ),
            manifest_endpoint(
                "event",
                format!("jobs/{job_type}/result"),
                job_result_key(service, job_type),
                JobResult::SCHEMA_NAME.to_owned(),
                carrying::<JobResult>(result_type),
            ),
            manifest_endpoint(
                "query",
                format!("jobs/{job_type}/history"),
                job_history_key(service, job_type),
                JobList::SCHEMA_NAME.to_owned(),
                JobList::SCHEMA.to_owned(),
            ),
        ]
    })
}
