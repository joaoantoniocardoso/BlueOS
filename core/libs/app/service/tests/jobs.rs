//! The Jobs surface every Service gets from the Kernel: submit, the ack, the controls, the `jobs` State, and the
//! Feedback, Job result and history of each Job type, through a real `build` (layer L3).

use core::{convert::Infallible, time::Duration};
use std::collections::BTreeMap;

use bytes::Bytes;
use tokio::time::timeout;

use blueos_api::{
    CommandAck, Message, cdr_encoding, command_key, info_query_key, job_feedback_key,
    job_history_key, job_result_key, jobs_key, query_key, status_state_key,
};
use blueos_comms::{QueryBody, Subscriber};
use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::{LevelRequest, LevelResponse, SetLevelGoal},
    blueos_msgs::{
        CommandAckStatus, JobFeedbackList, JobList, JobResult, JobStatusStatus, ServiceInfo,
        SettingsEnvelope,
    },
};
use blueos_jobs::{DomainJobs, JobControl, JobEnd, JobId, JobNature, JobStatus, Jobs};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

/// How long a brew takes to pour one cup once it executes.
const BREW_TIME: Duration = Duration::from_secs(60);
/// How many ended Jobs of each type the brewer keeps.
const RETENTION: usize = 2;
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
    /// Every brew that executed, kept after it ends so its Job result can name the cups it poured.
    brews: BTreeMap<JobId, Brew>,
}

#[derive(Clone)]
struct Brew {
    cups: u8,
    poured: u8,
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
            jobs: Jobs::with_retention(RETENTION),
            brews: BTreeMap::new(),
        })
        .command("Ping", |_: LevelRequest| Ok(BrewerRequest::Ping))
        .command("Refuse", |_: LevelRequest| Ok(BrewerRequest::Refuse))
        .job("Brew", BREW, |job_id, goal: SetLevelGoal| {
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
            |job_id, goal: SetLevelGoal| {
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
            |job_id, goal: SetLevelGoal| {
                Ok(BrewerRequest::Brew {
                    job_id,
                    cups: goal.level,
                })
            },
        )
        .job_feedback("Brew", |snapshot: &BrewerSnapshot, job_id| {
            let brew = snapshot.brews.get(&job_id)?;
            (brew.poured > 0).then_some(LevelResponse {
                level: brew.poured,
                max_level: brew.cups,
            })
        })
        .job_result("Brew", |snapshot: &BrewerSnapshot, job_id| {
            cups(snapshot.brews.get(&job_id).map_or(0, |brew| brew.poured))
        }))
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
            Command::Request(BrewerRequest::Brew { job_id, cups }) => {
                snapshot.brews.insert(job_id, Brew { cups, poured: 0 });
                return pour_next_cup(job_id);
            }
            Command::Request(BrewerRequest::Ping) => Ok(()),
            Command::Request(BrewerRequest::Refuse) => {
                return Outcome::Rejected {
                    reason: "refused".into(),
                };
            }
            // The brew follows the status a control set each time a cup is due.
            Command::Tick(job_id) => {
                if snapshot.jobs.job(job_id).map(|job| job.status) == Some(JobStatus::Canceling) {
                    snapshot.jobs.end(job_id, JobEnd::Canceled, "")
                } else {
                    let Some(brew) = snapshot.brews.get_mut(&job_id) else {
                        return Outcome::Rejected {
                            reason: "no such brew".into(),
                        };
                    };
                    brew.poured += 1;
                    if brew.poured < brew.cups {
                        return pour_next_cup(job_id);
                    }
                    snapshot.jobs.end(job_id, JobEnd::Succeeded, "")
                }
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

    let instant = harness.submit("Ping", ping, &LevelRequest::default()).await;
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
async fn a_service_without_settings_refuses_update_settings() {
    let harness = start().await;
    let job_id = JobId::from_u128(3);

    let ack = harness
        .submit("UpdateSettings", job_id, &SettingsEnvelope::default())
        .await;

    assert_eq!(
        ack,
        rejected(
            job_id,
            CommandAckStatus::StatusUnknown,
            "the Service has no settings"
        )
    );
    assert!(listed(&harness).await.is_empty());
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
        .submit("Refuse", job_id, &LevelRequest::default())
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
            cdr_encoding(SetLevelGoal::SCHEMA_NAME),
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
    next::<JobList>(&mut jobs).await;

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
        let body = QueryBody::new(goal.clone(), cdr_encoding(SetLevelGoal::SCHEMA_NAME))
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
async fn info_lists_the_jobs_state_and_the_feedback_result_and_history_of_each_job_type() {
    let harness = start().await;
    let service = BrewerService::NAME;

    let info = harness
        .query::<LevelRequest, ServiceInfo>("info", &LevelRequest::default())
        .await
        .expect("the info query answers");

    let listed: Vec<_> = info
        .endpoints
        .iter()
        .map(|endpoint| {
            (
                endpoint.kind.as_str(),
                endpoint.name.as_str(),
                endpoint.key.clone(),
                endpoint.interface_type.as_str(),
                endpoint.schema.as_str(),
            )
        })
        .collect();
    let (list, list_schema) = (JobList::SCHEMA_NAME, JobList::SCHEMA);
    assert_eq!(
        listed.len(),
        1 + 3 * 6,
        "jobs, then three per Job type: {listed:?}"
    );
    let shown: Vec<_> = listed
        .into_iter()
        .filter(|(_, name, ..)| {
            *name == "jobs"
                || name.starts_with("jobs/Brew/")
                || name.starts_with("jobs/UpdateSettings/")
        })
        .collect();
    assert_eq!(
        shown,
        [
            ("state", "jobs", jobs_key(service), list, list_schema),
            (
                "state",
                "jobs/Brew/feedback",
                job_feedback_key(service, "Brew"),
                LevelResponse::SCHEMA_NAME,
                LevelResponse::SCHEMA,
            ),
            (
                "event",
                "jobs/Brew/result",
                job_result_key(service, "Brew"),
                SetLevelGoal::SCHEMA_NAME,
                SetLevelGoal::SCHEMA,
            ),
            (
                "query",
                "jobs/Brew/history",
                job_history_key(service, "Brew"),
                list,
                list_schema
            ),
            (
                "state",
                "jobs/UpdateSettings/feedback",
                job_feedback_key(service, "UpdateSettings"),
                "",
                "",
            ),
            (
                "event",
                "jobs/UpdateSettings/result",
                job_result_key(service, "UpdateSettings"),
                "",
                ""
            ),
            (
                "query",
                "jobs/UpdateSettings/history",
                job_history_key(service, "UpdateSettings"),
                list,
                list_schema,
            ),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn the_feedback_state_holds_the_latest_feedback_of_a_job_until_it_ends() {
    let harness = start().await;
    let mut feedback = harness
        .backend()
        .subscribe(&job_feedback_key(BrewerService::NAME, "Brew"))
        .await
        .unwrap();
    let job_id = JobId::from_u128(7);
    harness.submit("Brew", job_id, &cups(3)).await;

    let first = next(&mut feedback).await;
    let second = next(&mut feedback).await;
    let latest = harness.job_feedback("Brew").await;
    let ended = next(&mut feedback).await;

    assert_eq!(fed_back(first), [(job_id, poured(1, 3))]);
    assert_eq!(fed_back(second), [(job_id, poured(2, 3))]);
    assert_eq!(fed_back(latest), [(job_id, poured(2, 3))]);
    assert_eq!(fed_back(ended), []);
    assert_eq!(
        listed(&harness).await,
        [entry(job_id, "Brew", JobStatusStatus::Succeeded, "")]
    );
}

#[tokio::test(start_paused = true)]
async fn a_client_that_opens_the_feedback_state_mid_job_sees_the_latest_feedback() {
    let harness = start().await;
    let [brewing, waiting] = [7, 8].map(JobId::from_u128);
    harness.submit("Brew", brewing, &cups(3)).await;
    harness.submit("Brew", waiting, &cups(3)).await;
    tokio::time::sleep(BREW_TIME * 2 + BREW_TIME / 2).await;

    let feedback = harness.job_feedback("Brew").await;

    assert_eq!(
        fed_back(feedback),
        [(brewing, poured(2, 3)), (waiting, poured(2, 3))]
    );
}

#[tokio::test(start_paused = true)]
async fn the_result_event_carries_how_a_job_ended_and_its_job_result() {
    let harness = start().await;
    let mut results = harness
        .backend()
        .subscribe(&job_result_key(BrewerService::NAME, "Brew"))
        .await
        .unwrap();
    let [aborted, succeeded, canceled] = [1, 2, 3].map(JobId::from_u128);

    harness.submit("Brew", aborted, &cups(0)).await;
    let abort = next(&mut results).await;
    harness.submit("Brew", succeeded, &cups(2)).await;
    let success = next(&mut results).await;
    harness.submit("Brew", canceled, &cups(2)).await;
    harness.control(canceled, JobControl::Cancel).await;
    let cancel = next(&mut results).await;

    assert_eq!(
        ended(abort),
        (
            entry(aborted, "Brew", JobStatusStatus::Aborted, "no cups to brew"),
            cups(0)
        )
    );
    assert_eq!(
        ended(success),
        (
            entry(succeeded, "Brew", JobStatusStatus::Succeeded, ""),
            cups(2)
        )
    );
    assert_eq!(
        ended(cancel),
        (
            entry(canceled, "Brew", JobStatusStatus::Canceled, ""),
            cups(0)
        )
    );
}

#[tokio::test(start_paused = true)]
async fn a_job_the_kernel_ends_publishes_its_result_too() {
    let harness = start().await;
    let mut results = harness
        .backend()
        .subscribe(&job_result_key(BrewerService::NAME, "Pour"))
        .await
        .unwrap();
    let job_id = JobId::from_u128(7);
    harness.submit("Pour", job_id, &cups(2)).await;

    harness
        .control(job_id, JobControl::AnswerPermission { granted: false })
        .await;
    let result: JobResult = next(&mut results).await;

    assert_eq!(
        row(result.job),
        entry(
            job_id,
            "Pour",
            JobStatusStatus::Canceled,
            "permission denied"
        )
    );
    assert_eq!(
        result.result,
        Vec::<u8>::new(),
        "Pour declares no Job result"
    );
}

#[tokio::test(start_paused = true)]
async fn the_history_query_returns_the_last_finished_jobs_of_a_type_in_order() {
    let harness = start().await;
    let [first, second, third, brew] = [1, 2, 3, 4].map(JobId::from_u128);
    for ping in [first, second, third] {
        harness.submit("Ping", ping, &LevelRequest::default()).await;
    }
    harness.submit("Brew", brew, &cups(0)).await;

    let pings = harness.job_history("Ping").await;
    let brews = harness.job_history("Brew").await;

    assert_eq!(
        pings.jobs.into_iter().map(row).collect::<Vec<_>>(),
        [
            entry(second, "Ping", JobStatusStatus::Succeeded, ""),
            entry(third, "Ping", JobStatusStatus::Succeeded, ""),
        ],
        "the last {RETENTION} Pings, oldest first"
    );
    assert_eq!(
        brews.jobs.into_iter().map(row).collect::<Vec<_>>(),
        [entry(
            brew,
            "Brew",
            JobStatusStatus::Aborted,
            "no cups to brew"
        )]
    );
}

async fn start() -> Harness<BrewerService> {
    Harness::start(()).await.unwrap()
}

fn cups(level: u8) -> SetLevelGoal {
    SetLevelGoal { level }
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
    harness.jobs().await.jobs.into_iter().map(row).collect()
}

/// One Job as the Kernel publishes it, in the form [`entry`] builds.
fn row(job: blueos_idl::msg::blueos_msgs::JobStatus) -> (JobId, String, JobStatusStatus, String) {
    (
        job.job_id.parse().unwrap(),
        job.job_type,
        job.status,
        job.reason,
    )
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

/// The next `M` the Service publishes on `subscriber`. Time is paused, so it advances to the next timer at once.
async fn next<M: Message>(subscriber: &mut Subscriber) -> M {
    let sample = timeout(Duration::from_secs(600), subscriber.recv())
        .await
        .unwrap()
        .unwrap();
    M::decode(&sample.payload().to_bytes()).unwrap()
}

/// Each Job in a `Brew` feedback State, with its decoded Feedback.
fn fed_back(list: JobFeedbackList) -> Vec<(JobId, LevelResponse)> {
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
fn poured(poured: u8, cups: u8) -> LevelResponse {
    LevelResponse {
        level: poured,
        max_level: cups,
    }
}

/// How a `Brew` ended, with its decoded Job result.
fn ended(result: JobResult) -> ((JobId, String, JobStatusStatus, String), SetLevelGoal) {
    (
        row(result.job),
        SetLevelGoal::decode(&result.result).unwrap(),
    )
}

/// Pours the next cup of the brew `job_id` after [`BREW_TIME`].
fn pour_next_cup(job_id: JobId) -> Decision<Brewer> {
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
fn status(jobs: &JobList) -> JobStatusStatus {
    let [job] = jobs.jobs.as_slice() else {
        panic!("expected one Job, got {jobs:?}");
    };
    job.status
}
