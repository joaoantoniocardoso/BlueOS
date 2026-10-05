//! Job result and repair feedback helpers.

use tokio::time::timeout;

use blueos_api::{Message, job_result_key};
use blueos_comms::Subscriber;
use blueos_idl::msg::{
    blueos_msgs::{JobFeedbackList, JobResult, JobStatus, JobStatusStatus},
    blueos_recorder_msgs::RepairRecordingFeedback,
};
use blueos_jobs::JobId;
use blueos_recorder_app::RecorderService;
use blueos_service::{Service, testing::Harness};

use super::super::STATE_WAIT_TIMEOUT;

/// Subscribes to the Job result Event of the Job type `job_type`.
pub(crate) async fn subscribe_job_results(
    harness: &Harness<RecorderService>,
    job_type: &str,
) -> Subscriber {
    harness
        .backend()
        .subscribe(&job_result_key(RecorderService::NAME, job_type))
        .await
        .expect("subscribe to the Job results")
}

/// The next Job result on `results`: how its Job ended, and the Job result as an `R`.
pub(crate) async fn next_job_result<R: Message>(results: &mut Subscriber) -> (JobStatus, R) {
    let sample = timeout(STATE_WAIT_TIMEOUT, results.recv())
        .await
        .expect("a Job result")
        .expect("the Job result subscription is open");
    let message = JobResult::decode(&sample.payload().to_bytes()).expect("decode JobResult");
    (
        message.job,
        R::decode(&message.result).expect("decode the Job result"),
    )
}

pub(crate) async fn repair_job_status(
    harness: &Harness<RecorderService>,
    job_id: JobId,
) -> JobStatusStatus {
    harness
        .jobs()
        .await
        .expect("jobs state")
        .jobs
        .into_iter()
        .find(|job| job.job_id == job_id.to_string())
        .expect("repair job is listed")
        .status
}

pub(crate) fn read_offsets(feedback: &JobFeedbackList, job_id: JobId) -> Option<(u64, u64)> {
    feedback
        .jobs
        .iter()
        .find(|job| job.job_id == job_id.to_string())
        .and_then(|job| {
            RepairRecordingFeedback::decode(&job.feedback)
                .ok()
                .map(|decoded| (decoded.bytes_processed, decoded.total_bytes))
        })
}

pub(crate) async fn wait_for_read_offset(subscriber: &mut Subscriber, job_id: JobId, offset: u64) {
    timeout(STATE_WAIT_TIMEOUT, async {
        while let Some(sample) = subscriber.recv().await {
            let feedback =
                JobFeedbackList::decode(&sample.payload().to_bytes()).expect("decode feedback");
            if read_offsets(&feedback, job_id).is_some_and(|(read, _total)| read >= offset) {
                return;
            }
        }
        panic!("repair feedback stream closed");
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for repair read offset {offset}"));
}
