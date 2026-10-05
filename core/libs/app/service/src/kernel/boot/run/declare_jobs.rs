//! Declares per-job-type states and the aggregate jobs state.

use std::sync::Arc;

use tokio::sync::watch;

use blueos_api::{job_feedback_key, job_history_key, jobs_key};
use blueos_comms::CommsBackend;
use blueos_domain::Domain;

use crate::{builder::ServiceBuilder, service::ServiceError};

use super::super::{
    super::{
        endpoints::declare,
        types::{IntoInput, JobTypeOutput, Kernel, OutputWithoutJobType},
    },
    commands::declare_control_endpoints,
};

pub(super) type PendingJobOutput = (
    blueos_comms::Queryable,
    String,
    watch::Receiver<Option<bytes::Bytes>>,
    blueos_comms::Queryable,
    String,
    watch::Receiver<Option<bytes::Bytes>>,
);

pub(super) struct JobEndpointsBootPrepared<D: Domain> {
    pub job_outputs: Vec<JobTypeOutput<D>>,
    pub pending_job_outputs: Vec<PendingJobOutput>,
    pub jobs_latest: watch::Sender<Option<bytes::Bytes>>,
    pub jobs_queryable: blueos_comms::Queryable,
    pub pending_commands: Vec<(blueos_comms::Queryable, IntoInput<D>)>,
}

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    pub(super) async fn declare_job_endpoints_at_boot(
        service: &'static str,
        backend: Arc<dyn CommsBackend>,
        builder: &mut ServiceBuilder<D, Context>,
        job_type_names: Vec<String>,
        mut pending_commands: Vec<(blueos_comms::Queryable, IntoInput<D>)>,
    ) -> Result<JobEndpointsBootPrepared<D>, ServiceError> {
        let mut job_outputs = Vec::new();
        let mut pending_job_outputs = Vec::new();
        for job_type in job_type_names {
            let feedback_key = job_feedback_key(service, &job_type);
            let history_key = job_history_key(service, &job_type);
            let job_output = JobTypeOutput {
                output: builder.job_outputs.remove(&job_type).unwrap_or_default(),
                job_type,
                feedback_latest: watch::Sender::new(None),
                history: watch::Sender::new(None),
            };
            let feedback_queryable = declare(&*backend, &feedback_key).await?;
            let history_queryable = declare(&*backend, &history_key).await?;
            pending_job_outputs.push((
                feedback_queryable,
                feedback_key,
                job_output.feedback_latest.subscribe(),
                history_queryable,
                history_key,
                job_output.history.subscribe(),
            ));
            job_outputs.push(job_output);
        }
        if let Some(job_type) = builder.job_outputs.keys().next() {
            return Err(ServiceError::Build(Box::new(OutputWithoutJobType {
                job_type: job_type.clone(),
            })));
        }
        pending_commands.extend(declare_control_endpoints(service, backend.as_ref()).await?);
        let jobs_queryable = declare(&*backend, jobs_key(service)).await?;
        let jobs_latest = watch::Sender::new(None);
        Ok(JobEndpointsBootPrepared {
            job_outputs,
            pending_job_outputs,
            jobs_latest,
            jobs_queryable,
            pending_commands,
        })
    }
}
