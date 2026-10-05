//! Shared helpers for brewer jobs integration tests.

use core::time::Duration;

use tokio::time::timeout;

use blueos_api::{
    CommandAck, Message, command_key, job_feedback_key, job_history_key, job_result_key, jobs_key,
};
use blueos_comms::{QueryBody, Subscriber};
use blueos_domain::{Decision, Effect, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::{LevelResponse, SetLevelGoal},
    blueos_msgs::{
        CommandAckStatus, JobFeedbackList, JobList, JobResult, JobStatusStatus, ServiceInfo,
        UpdateSettingsFeedback, UpdateSettingsResult,
    },
};
use blueos_jobs::JobId;
use blueos_service::{Service, testing::Harness};

use super::{BREW_TIME, Brewer, BrewerService};

type InfoEndpointRow = (String, String, String, String, String);

pub async fn start() -> Harness<BrewerService> {
    Harness::start(()).await.unwrap()
}

pub fn cups(level: u8) -> SetLevelGoal {
    SetLevelGoal { level }
}

pub fn accepted(job_id: JobId, status: CommandAckStatus) -> CommandAck {
    CommandAck {
        accepted: true,
        job_id: job_id.to_string(),
        status,
        reason: String::new(),
    }
}

pub fn rejected(job_id: JobId, status: CommandAckStatus, reason: &str) -> CommandAck {
    CommandAck {
        accepted: false,
        job_id: job_id.to_string(),
        status,
        reason: reason.to_owned(),
    }
}

pub async fn get_ack(harness: &Harness<BrewerService>, key: &str, body: QueryBody) -> CommandAck {
    let replies = harness
        .backend()
        .get(key, Some(body), Duration::from_secs(10))
        .await
        .unwrap();
    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one ack, got {replies:?}");
    };
    CommandAck::decode(&reply.payload().to_bytes()).unwrap()
}

/// Every Job in the `jobs` State, as a late client reads it.
pub async fn listed(
    harness: &Harness<BrewerService>,
) -> Vec<(JobId, String, JobStatusStatus, String)> {
    harness
        .jobs()
        .await
        .unwrap()
        .jobs
        .into_iter()
        .map(row)
        .collect()
}

/// How a `Brew` ended, with its decoded Job result.
pub fn ended(result: JobResult) -> ((JobId, String, JobStatusStatus, String), SetLevelGoal) {
    (
        row(result.job),
        SetLevelGoal::decode(&result.result).unwrap(),
    )
}

/// One Job as the Kernel publishes it, in the form [`entry`] builds.
pub fn row(
    job: blueos_idl::msg::blueos_msgs::JobStatus,
) -> (JobId, String, JobStatusStatus, String) {
    (
        job.job_id.parse().unwrap(),
        job.job_type,
        job.status,
        job.reason,
    )
}

pub fn entry(
    job_id: JobId,
    job_type: &str,
    status: JobStatusStatus,
    reason: &str,
) -> (JobId, String, JobStatusStatus, String) {
    (job_id, job_type.to_owned(), status, reason.to_owned())
}

pub async fn subscribe(harness: &Harness<BrewerService>) -> Subscriber {
    harness
        .backend()
        .subscribe(&jobs_key(BrewerService::NAME))
        .await
        .unwrap()
}

/// The next `M` the Service publishes on `subscriber`. Time is paused, so it advances to the next timer at once.
pub async fn next<M: Message>(subscriber: &mut Subscriber) -> M {
    let sample = timeout(Duration::from_secs(600), subscriber.recv())
        .await
        .unwrap()
        .unwrap();
    M::decode(&sample.payload().to_bytes()).unwrap()
}

/// Each Job in a `Brew` feedback State, with its decoded Feedback.
pub fn fed_back(list: JobFeedbackList) -> Vec<(JobId, LevelResponse)> {
    list.jobs
        .into_iter()
        .map(|job| {
            (
                job.job_id.parse().unwrap(),
                LevelResponse::decode(&job.feedback).unwrap(),
            )
        })
        .collect()
}

/// The Feedback of a brew that poured `poured` of its `cups`.
pub fn poured(poured: u8, cups: u8) -> LevelResponse {
    LevelResponse {
        level: poured,
        max_level: cups,
    }
}

/// Pours the next cup of the brew `job_id` after [`BREW_TIME`].
pub fn pour_next_cup(job_id: JobId) -> Decision<Brewer> {
    Outcome::Applied {
        events: Vec::new(),
        effects: vec![Effect::Schedule {
            after: BREW_TIME,
            key: job_id,
            command: job_id,
        }],
    }
}

/// The status of the only Job in `jobs`.
pub fn status(jobs: &JobList) -> JobStatusStatus {
    let [job] = jobs.jobs.as_slice() else {
        panic!("expected one Job, got {jobs:?}");
    };
    job.status
}

pub fn info_endpoint_rows(info: &ServiceInfo) -> Vec<InfoEndpointRow> {
    info.endpoints
        .iter()
        .map(|endpoint| {
            (
                endpoint.kind.as_str().to_owned(),
                endpoint.name.as_str().to_owned(),
                endpoint.key.clone(),
                endpoint.interface_type.as_str().to_owned(),
                endpoint.schema.as_str().to_owned(),
            )
        })
        .collect()
}

fn wrapped_message_schema(wrapper: &str, part: &str, part_schema: &str) -> String {
    const SEPARATOR: &str =
        "================================================================================";
    format!("{wrapper}\n{SEPARATOR}\nMSG: {part}\n{part_schema}")
}

fn brew_job_info_rows(service: &str, list: &str, list_schema: &str) -> Vec<InfoEndpointRow> {
    let (feedback_list, result) = (JobFeedbackList::SCHEMA_NAME, JobResult::SCHEMA_NAME);
    let brew_feedback = wrapped_message_schema(
        JobFeedbackList::SCHEMA,
        "blueos_example_msgs/Level_Response",
        LevelResponse::SCHEMA,
    );
    let brew_result = wrapped_message_schema(
        JobResult::SCHEMA,
        "blueos_example_msgs/SetLevel_Goal",
        SetLevelGoal::SCHEMA,
    );
    vec![
        (
            "state".into(),
            "jobs/Brew/feedback".into(),
            job_feedback_key(service, "Brew"),
            feedback_list.into(),
            brew_feedback,
        ),
        (
            "event".into(),
            "jobs/Brew/result".into(),
            job_result_key(service, "Brew"),
            result.into(),
            brew_result,
        ),
        (
            "query".into(),
            "jobs/Brew/history".into(),
            job_history_key(service, "Brew"),
            list.into(),
            list_schema.into(),
        ),
    ]
}

fn update_settings_action_and_jobs_list(
    service: &str,
    list: &str,
    list_schema: &str,
) -> Vec<InfoEndpointRow> {
    vec![
        (
            "job".into(),
            "UpdateSettings".into(),
            command_key(service, "UpdateSettings"),
            "blueos_msgs/action/UpdateSettings".into(),
            blueos_idl::schema("blueos_msgs/action/UpdateSettings")
                .unwrap()
                .into(),
        ),
        (
            "state".into(),
            "jobs".into(),
            jobs_key(service),
            list.into(),
            list_schema.into(),
        ),
    ]
}

fn update_settings_job_endpoint_rows(
    service: &str,
    list: &str,
    list_schema: &str,
) -> Vec<InfoEndpointRow> {
    let (feedback_list, result) = (JobFeedbackList::SCHEMA_NAME, JobResult::SCHEMA_NAME);
    let settings_feedback = wrapped_message_schema(
        JobFeedbackList::SCHEMA,
        "blueos_msgs/UpdateSettings_Feedback",
        UpdateSettingsFeedback::SCHEMA,
    );
    let settings_result = wrapped_message_schema(
        JobResult::SCHEMA,
        "blueos_msgs/UpdateSettings_Result",
        UpdateSettingsResult::SCHEMA,
    );
    vec![
        (
            "state".into(),
            "jobs/UpdateSettings/feedback".into(),
            job_feedback_key(service, "UpdateSettings"),
            feedback_list.into(),
            settings_feedback,
        ),
        (
            "event".into(),
            "jobs/UpdateSettings/result".into(),
            job_result_key(service, "UpdateSettings"),
            result.into(),
            settings_result,
        ),
        (
            "query".into(),
            "jobs/UpdateSettings/history".into(),
            job_history_key(service, "UpdateSettings"),
            list.into(),
            list_schema.into(),
        ),
    ]
}

pub fn expected_brewer_job_info_rows(service: &str) -> Vec<InfoEndpointRow> {
    let (list, list_schema) = (JobList::SCHEMA_NAME, JobList::SCHEMA);
    let mut rows = update_settings_action_and_jobs_list(service, list, list_schema);
    rows.extend(brew_job_info_rows(service, list, list_schema));
    rows.extend(update_settings_job_endpoint_rows(
        service,
        list,
        list_schema,
    ));
    rows
}
