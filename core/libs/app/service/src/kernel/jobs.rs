//! Job output encoding and Jobs table access.

use blueos_api::Message;
use blueos_domain::Domain;
use blueos_idl::msg::blueos_msgs::{JobFeedback, JobFeedbackList, JobList, JobResult};
use blueos_jobs::Jobs;

use crate::builder::{JobsAccess, job_status};

use super::types::{EncodedJobOutput, JobTypeOutput};

/// Encodes each Job type's Feedback State and history from `snapshot` and `jobs`, and the Job result of each Job
/// that ended since `before`.
pub(crate) fn encode_job_outputs<D: Domain>(
    job_outputs: &[JobTypeOutput<D>],
    snapshot: &D::Snapshot,
    jobs: &Jobs,
    before: &Jobs,
) -> Vec<EncodedJobOutput> {
    job_outputs
        .iter()
        .map(|job_output| {
            let of_type = || {
                jobs.list()
                    .filter(|job| job.job_type == job_output.job_type)
            };
            let fed_back = job_output.output.feedback.as_ref().map_or_else(
                || Ok(Vec::new()),
                |project| {
                    of_type()
                        .filter(|job| !job.status.has_ended())
                        .filter_map(|job| {
                            let encoded = project(snapshot, job.job_id)?;
                            Some(encoded.map(|feedback| JobFeedback {
                                job_id: job.job_id.to_string(),
                                feedback,
                            }))
                        })
                        .collect()
                },
            );
            let history = JobList {
                jobs: of_type()
                    .filter(|job| job.status.has_ended())
                    .map(job_status)
                    .collect(),
            };
            let results = of_type()
                .filter(|job| {
                    job.status.has_ended()
                        && !before
                            .job(job.job_id)
                            .is_some_and(|earlier| earlier.status.has_ended())
                })
                .map(|job| {
                    let result = job_output
                        .output
                        .result
                        .as_ref()
                        .map_or_else(|| Ok(Vec::new()), |result| result(snapshot, job.job_id))?;
                    JobResult {
                        job: job_status(job),
                        result,
                    }
                    .encode()
                })
                .collect();
            EncodedJobOutput {
                feedback: fed_back.and_then(|entries| JobFeedbackList { jobs: entries }.encode()),
                history: history.encode(),
                results,
            }
        })
        .collect()
}

/// The Jobs, in the Snapshot when `jobs_access` says the Domain keeps them there, else in `own_jobs`.
pub(crate) fn jobs_in<'jobs, D: Domain>(
    jobs_access: Option<JobsAccess<D>>,
    snapshot: &'jobs D::Snapshot,
    own_jobs: &'jobs Jobs,
) -> &'jobs Jobs {
    match jobs_access {
        Some((jobs, _jobs_mut)) => jobs(snapshot),
        None => own_jobs,
    }
}

/// The Jobs, in the Snapshot when `jobs_access` says the Domain keeps them there, else in `own_jobs`.
pub(crate) fn jobs_in_mut<'jobs, D: Domain>(
    jobs_access: Option<JobsAccess<D>>,
    snapshot: &'jobs mut D::Snapshot,
    own_jobs: &'jobs mut Jobs,
) -> &'jobs mut Jobs {
    match jobs_access {
        Some((_jobs, jobs_mut)) => jobs_mut(snapshot),
        None => own_jobs,
    }
}
