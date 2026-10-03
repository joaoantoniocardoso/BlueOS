//! The Jobs surface every Service gets from the Kernel: submit, the ack, the controls and the `jobs` State, through a
//! real `build` (layer L3).

use core::{convert::Infallible, time::Duration};

use bytes::Bytes;
use tokio::time::timeout;

use blueos_api::{
    CommandAck, Message, cdr_encoding, command_key, info_query_key, jobs_key, query_key,
    status_state_key,
};
use blueos_comms::{QueryBody, Subscriber};
use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::{EmptyRequest, SetLevelRequest},
    blueos_msgs::{CommandAckStatus, JobList, JobStatusStatus, ServiceInfo},
};
use blueos_jobs::{DomainJobs, JobControl, JobEnd, JobId, JobNature, JobStatus, Jobs};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

/// How long a brew takes once it executes.
const BREW_TIME: Duration = Duration::from_secs(60);
/// A Job type that runs until its time is up, and that a client may cancel, pause and resume.
const BREW: JobNature = JobNature {
    lasting: true,
    cancellable: true,
    pausable: true,
    ..JobNature::INSTANT
};

struct BrewerService;

#[derive(Clone)]
struct BrewerSnapshot {
    jobs: Jobs,
}

enum BrewerRequest {
    /// Brews `cups` cups. No cups aborts the Job at once.
    Brew { job_id: JobId, cups: u8 },
    /// Starts nothing.
    Ping,
    /// The Domain rejects it.
    Refuse,
}

struct Brewer;

impl Service for BrewerService {
    type Domain = Brewer;
    type Context = ();
    type Arguments = ();

    const NAME: &'static str = "brewer";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<()>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<()>,
        _context: &(),
    ) -> Result<ServiceBuilder<Brewer>, ServiceError> {
        Ok(ServiceBuilder::new(BrewerSnapshot {
            jobs: Jobs::default(),
        })
        .command("Ping", |_: EmptyRequest| Ok(BrewerRequest::Ping))
        .command("Refuse", |_: EmptyRequest| Ok(BrewerRequest::Refuse))
        .job("Brew", BREW, |job_id, goal: SetLevelRequest| {
            Ok(BrewerRequest::Brew {
                job_id,
                cups: goal.level,
            })
        })
        .job(
            "Steep",
            JobNature {
                lasting: true,
                ..JobNature::INSTANT
            },
            |job_id, goal: SetLevelRequest| {
                Ok(BrewerRequest::Brew {
                    job_id,
                    cups: goal.level,
                })
            },
        )
        .job(
            "Pour",
            JobNature {
                needs_permission: true,
                ..BREW
            },
            |job_id, goal: SetLevelRequest| {
                Ok(BrewerRequest::Brew {
                    job_id,
                    cups: goal.level,
                })
            },
        ))
    }
}

impl Domain for Brewer {
    type Snapshot = BrewerSnapshot;
    type Request = BrewerRequest;
    type IoResult = Infallible;
    type Tick = JobId;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = Infallible;
    type TimerKey = JobId;

    fn handle(
        snapshot: &mut BrewerSnapshot,
        command: Command<BrewerRequest, Infallible, JobId, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let ended = match command {
            Command::Request(BrewerRequest::Brew { job_id, cups: 0 }) => {
                snapshot
                    .jobs
                    .end(job_id, JobEnd::Aborted, "no cups to brew")
            }
            Command::Request(BrewerRequest::Brew { job_id, .. }) => {
                return Outcome::Applied {
                    events: Vec::new(),
                    effects: vec![Effect::Schedule {
                        after: BREW_TIME,
                        key: job_id,
                        command: job_id,
                    }],
                };
            }
            Command::Request(BrewerRequest::Ping) => Ok(()),
            Command::Request(BrewerRequest::Refuse) => {
                return Outcome::Rejected {
                    reason: "refused".into(),
                };
            }
            // The brew follows the status a control set when its time is up.
            Command::Tick(job_id) => {
                let end = match snapshot.jobs.job(job_id).map(|job| job.status) {
                    Some(JobStatus::Canceling) => JobEnd::Canceled,
                    _ => JobEnd::Succeeded,
                };
                snapshot.jobs.end(job_id, end, "")
            }
            Command::IoResult(result) => match result {},
            Command::ObservedFact(fact) => match fact {},
        };
        match ended {
            Ok(()) => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
            Err(error) => Outcome::Rejected {
                reason: Box::new(error),
            },
        }
    }

    fn io_failed(
        request: Infallible,
        _error: IoError,
    ) -> Command<BrewerRequest, Infallible, JobId, Infallible> {
        match request {}
    }
}

impl DomainJobs for Brewer {
    fn jobs(snapshot: &BrewerSnapshot) -> &Jobs {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut BrewerSnapshot) -> &mut Jobs {
        &mut snapshot.jobs
    }
}

#[tokio::test(start_paused = true)]
async fn a_submit_acks_the_client_job_id_and_the_job_executes() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);

    let ack = harness.submit("Brew", job_id, &cups(2)).await;

    assert_eq!(ack, accepted(job_id, CommandAckStatus::Executing));
    assert_eq!(
        listed(&harness).await,
        [entry(job_id, "Brew", JobStatusStatus::Executing, "")]
    );
}

#[tokio::test(start_paused = true)]
async fn a_job_that_ends_in_the_step_that_accepts_it_acks_its_final_status() {
    let harness = start().await;
    let [ping, empty] = [1, 2].map(JobId::from_u128);

    let instant = harness.submit("Ping", ping, &EmptyRequest::default()).await;
    let aborted = harness.submit("Brew", empty, &cups(0)).await;

    assert_eq!(instant, accepted(ping, CommandAckStatus::Succeeded));
    assert_eq!(
        aborted,
        CommandAck {
            reason: "no cups to brew".to_owned(),
            ..accepted(empty, CommandAckStatus::Aborted)
        }
    );
    assert_eq!(
        listed(&harness).await,
        [
            entry(ping, "Ping", JobStatusStatus::Succeeded, ""),
            entry(empty, "Brew", JobStatusStatus::Aborted, "no cups to brew"),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn the_same_id_and_goal_is_a_retry_and_runs_the_job_once() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);
    harness.submit("Brew", job_id, &cups(2)).await;

    let retry = harness.submit("Brew", job_id, &cups(2)).await;

    assert_eq!(retry, accepted(job_id, CommandAckStatus::Executing));
    assert_eq!(listed(&harness).await.len(), 1);
}

#[tokio::test(start_paused = true)]
async fn the_same_id_with_another_goal_is_rejected_as_reused() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);
    harness.submit("Brew", job_id, &cups(2)).await;

    let reused = harness.submit("Brew", job_id, &cups(3)).await;
    let other_type = harness.submit("Steep", job_id, &cups(2)).await;

    for ack in [reused, other_type] {
        assert_eq!(
            ack,
            rejected(job_id, CommandAckStatus::StatusUnknown, "id reused")
        );
    }
    assert_eq!(
        listed(&harness).await,
        [entry(job_id, "Brew", JobStatusStatus::Executing, "")]
    );
}

#[tokio::test(start_paused = true)]
async fn a_rejected_submit_leaves_no_job() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);

    let refused = harness
        .submit("Refuse", job_id, &EmptyRequest::default())
        .await;

    assert_eq!(
        refused,
        rejected(job_id, CommandAckStatus::StatusUnknown, "refused")
    );
    assert_eq!(harness.jobs().await, JobList::default());
}

#[tokio::test(start_paused = true)]
async fn a_command_without_a_job_id_is_rejected() {
    let harness = start().await;

    let mut acks = Vec::new();
    for attachment in [None, Some("not a uuid")] {
        let mut body = QueryBody::new(
            cups(2).encode().unwrap(),
            cdr_encoding(SetLevelRequest::SCHEMA_NAME),
        );
        if let Some(attachment) = attachment {
            body = body.with_attachment(Bytes::from_static(attachment.as_bytes()));
        }
        acks.push(get_ack(&harness, &command_key(BrewerService::NAME, "Brew"), body).await);
    }

    for ack in acks {
        assert!(!ack.accepted);
        assert_eq!(ack.job_id, "");
        assert_eq!(ack.reason, "the Command's attachment is not a Job id");
    }
    assert_eq!(harness.jobs().await, JobList::default());
}

#[tokio::test(start_paused = true)]
async fn cancel_pause_and_resume_drive_a_job_through_its_lifecycle() {
    let harness = start().await;
    let mut jobs = subscribe(&harness).await;
    let job_id = JobId::from_u128(7);
    harness.submit("Brew", job_id, &cups(2)).await;
    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Executing);

    let paused = harness.control(job_id, JobControl::Pause).await;
    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Paused);
    let resumed = harness.control(job_id, JobControl::Resume).await;
    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Executing);
    let canceled = harness.control(job_id, JobControl::Cancel).await;
    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Canceling);

    assert_eq!(paused, accepted(job_id, CommandAckStatus::Paused));
    assert_eq!(resumed, accepted(job_id, CommandAckStatus::Executing));
    assert_eq!(canceled, accepted(job_id, CommandAckStatus::Canceling));
    assert_eq!(
        status(&next(&mut jobs).await),
        JobStatusStatus::Canceled,
        "the brew ends Canceled when its time is up"
    );
}

#[tokio::test(start_paused = true)]
async fn a_lasting_job_succeeds_when_its_domain_ends_it() {
    let harness = start().await;
    let mut jobs = subscribe(&harness).await;
    harness.submit("Brew", JobId::from_u128(7), &cups(2)).await;
    next(&mut jobs).await;

    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Succeeded);
}

#[tokio::test(start_paused = true)]
async fn a_control_the_job_type_does_not_allow_is_rejected() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);
    harness.submit("Steep", job_id, &cups(2)).await;

    for control in [
        JobControl::Cancel,
        JobControl::Pause,
        JobControl::Resume,
        JobControl::AnswerPermission { granted: true },
    ] {
        assert_eq!(
            harness.control(job_id, control).await,
            rejected(
                job_id,
                CommandAckStatus::Executing,
                &format!("Steep does not allow {control}")
            )
        );
    }
    assert_eq!(
        listed(&harness).await,
        [entry(job_id, "Steep", JobStatusStatus::Executing, "")]
    );
}

#[tokio::test(start_paused = true)]
async fn a_control_that_does_not_apply_to_the_status_is_rejected() {
    let harness = start().await;
    let [brewing, unknown] = [7, 8].map(JobId::from_u128);
    harness.submit("Brew", brewing, &cups(2)).await;

    let resume = harness.control(brewing, JobControl::Resume).await;
    harness.control(brewing, JobControl::Cancel).await;
    let pause = harness.control(brewing, JobControl::Pause).await;
    let missing = harness.control(unknown, JobControl::Cancel).await;

    assert_eq!(resume, accepted(brewing, CommandAckStatus::Executing));
    assert_eq!(
        pause,
        rejected(
            brewing,
            CommandAckStatus::Canceling,
            &format!("PauseJob does not apply to the Job {brewing}, which is Canceling")
        )
    );
    assert_eq!(
        missing,
        rejected(
            unknown,
            CommandAckStatus::StatusUnknown,
            &format!("there is no Job {unknown}")
        )
    );
}

#[tokio::test(start_paused = true)]
async fn a_job_waits_for_permission_and_executes_once_it_is_granted() {
    let harness = start().await;
    let mut jobs = subscribe(&harness).await;
    let job_id = JobId::from_u128(7);

    let submitted = harness.submit("Pour", job_id, &cups(2)).await;
    assert_eq!(
        status(&next(&mut jobs).await),
        JobStatusStatus::WaitingForPermission
    );
    let granted = harness
        .control(job_id, JobControl::AnswerPermission { granted: true })
        .await;
    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Executing);

    assert_eq!(
        submitted,
        accepted(job_id, CommandAckStatus::WaitingForPermission)
    );
    assert_eq!(granted, accepted(job_id, CommandAckStatus::Executing));
    assert_eq!(
        status(&next(&mut jobs).await),
        JobStatusStatus::Succeeded,
        "the Domain received the Goal once permission was granted"
    );
}

#[tokio::test(start_paused = true)]
async fn a_denied_permission_cancels_the_job() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);
    harness.submit("Pour", job_id, &cups(2)).await;

    let denied = harness
        .control(job_id, JobControl::AnswerPermission { granted: false })
        .await;

    assert_eq!(
        denied,
        CommandAck {
            reason: "permission denied".to_owned(),
            ..accepted(job_id, CommandAckStatus::Canceled)
        }
    );
    assert_eq!(
        listed(&harness).await,
        [entry(
            job_id,
            "Pour",
            JobStatusStatus::Canceled,
            "permission denied"
        )]
    );
}

#[tokio::test(start_paused = true)]
async fn no_key_outside_command_changes_anything() {
    let harness = start().await;
    let goal = cups(2).encode().unwrap();
    let service = BrewerService::NAME;
    let keys = [
        jobs_key(service),
        info_query_key(service),
        status_state_key(service),
        query_key(service, "Brew"),
        format!("blueos/v1/{service}/Brew"),
        format!("blueos/v1/{service}/CancelJob"),
    ];

    for key in &keys {
        let body = QueryBody::new(goal.clone(), cdr_encoding(SetLevelRequest::SCHEMA_NAME))
            .with_attachment(Bytes::from(JobId::from_u128(7).to_string()));
        drop(
            harness
                .backend()
                .get(key, Some(body), Duration::from_secs(1))
                .await,
        );
    }

    assert_eq!(harness.jobs().await, JobList::default());
}

#[tokio::test(start_paused = true)]
async fn info_lists_the_jobs_state() {
    let harness = start().await;

    let info = harness
        .query::<EmptyRequest, ServiceInfo>("info", &EmptyRequest::default())
        .await
        .expect("the info query answers");

    let [jobs] = info.endpoints.as_slice() else {
        panic!("expected only the jobs State, got {:?}", info.endpoints);
    };
    assert_eq!(jobs.kind, "state");
    assert_eq!(jobs.name, "jobs");
    assert_eq!(jobs.key, jobs_key(BrewerService::NAME));
    assert_eq!(jobs.request_schema, "");
    assert_eq!(jobs.response_schema, JobList::SCHEMA_NAME);
}

async fn start() -> Harness<BrewerService> {
    Harness::start(()).await.unwrap()
}

fn cups(level: u8) -> SetLevelRequest {
    SetLevelRequest { level }
}

fn accepted(job_id: JobId, status: CommandAckStatus) -> CommandAck {
    CommandAck {
        accepted: true,
        job_id: job_id.to_string(),
        status,
        reason: String::new(),
    }
}

fn rejected(job_id: JobId, status: CommandAckStatus, reason: &str) -> CommandAck {
    CommandAck {
        accepted: false,
        job_id: job_id.to_string(),
        status,
        reason: reason.to_owned(),
    }
}

async fn get_ack(harness: &Harness<BrewerService>, key: &str, body: QueryBody) -> CommandAck {
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
async fn listed(harness: &Harness<BrewerService>) -> Vec<(JobId, String, JobStatusStatus, String)> {
    harness
        .jobs()
        .await
        .jobs
        .into_iter()
        .map(|job| {
            (
                job.job_id.parse().unwrap(),
                job.job_type,
                job.status,
                job.reason,
            )
        })
        .collect()
}

fn entry(
    job_id: JobId,
    job_type: &str,
    status: JobStatusStatus,
    reason: &str,
) -> (JobId, String, JobStatusStatus, String) {
    (job_id, job_type.to_owned(), status, reason.to_owned())
}

async fn subscribe(harness: &Harness<BrewerService>) -> Subscriber {
    harness
        .backend()
        .subscribe(&jobs_key(BrewerService::NAME))
        .await
        .unwrap()
}

/// The next `jobs` State the Service publishes. Time is paused, so it advances to the next timer at once.
async fn next(jobs: &mut Subscriber) -> JobList {
    let sample = timeout(Duration::from_secs(600), jobs.recv())
        .await
        .unwrap()
        .unwrap();
    JobList::decode(&sample.payload().to_bytes()).unwrap()
}

/// The status of the only Job in `jobs`.
fn status(jobs: &JobList) -> JobStatusStatus {
    let [job] = jobs.jobs.as_slice() else {
        panic!("expected one Job, got {jobs:?}");
    };
    job.status
}
