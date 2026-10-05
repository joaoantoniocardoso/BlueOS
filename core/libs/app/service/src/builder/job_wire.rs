//! Job list wire encoding for the `jobs` State.

use blueos_idl::msg::blueos_msgs::{JobList, JobStatus, JobStatusStatus};
use blueos_jobs::{Job, Jobs};

pub(crate) fn job_list(jobs: &Jobs) -> JobList {
    JobList {
        jobs: jobs.list().map(job_status).collect(),
    }
}

/// One Job as clients see it in the `jobs` State, a Job result and a history.
pub(crate) fn job_status(job: &Job) -> JobStatus {
    JobStatus {
        job_id: job.job_id.to_string(),
        job_type: job.job_type.clone(),
        status: wire_status(job.status),
        reason: job.reason.clone(),
    }
}

/// The status of `blueos_msgs/JobStatus` for a Job's status; `blueos_msgs/CommandAck` shares its `STATUS_` values.
// qual:allow(dry, boilerplate) reason: "JobStatusStatus mirrors blueos_jobs::JobStatus for CDR wire encoding"
pub(crate) const fn wire_status(status: blueos_jobs::JobStatus) -> JobStatusStatus {
    match status {
        blueos_jobs::JobStatus::Accepted => JobStatusStatus::Accepted,
        blueos_jobs::JobStatus::Executing => JobStatusStatus::Executing,
        blueos_jobs::JobStatus::Canceling => JobStatusStatus::Canceling,
        blueos_jobs::JobStatus::Succeeded => JobStatusStatus::Succeeded,
        blueos_jobs::JobStatus::Canceled => JobStatusStatus::Canceled,
        blueos_jobs::JobStatus::Aborted => JobStatusStatus::Aborted,
        blueos_jobs::JobStatus::WaitingForPermission => JobStatusStatus::WaitingForPermission,
        blueos_jobs::JobStatus::WaitingForResource => JobStatusStatus::WaitingForResource,
        blueos_jobs::JobStatus::Paused => JobStatusStatus::Paused,
    }
}
