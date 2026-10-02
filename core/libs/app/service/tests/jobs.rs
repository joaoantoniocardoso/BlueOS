//! The standard `jobs` State and the root `job_id` in the ack, through a real `build` (layer L3).

use core::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
    time::Duration,
};

use tokio::time::timeout;

use blueos_api::{JOB_ID_NONE, Message, jobs_key};
use blueos_comms::Subscriber;
use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::EmptyRequest,
    blueos_msgs::{JobList, JobStatusStatus, ServiceInfo},
};
use blueos_jobs::{DomainJobs, JobEnd, JobGraph, JobId, JobStatus, Jobs, LeafJob};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct BrewerService;

#[derive(clap::Args)]
struct BrewerArguments {
    #[arg(long, default_value_t = 16)]
    retention: usize,
}

#[derive(Clone)]
struct BrewerSnapshot {
    jobs: Jobs<Step>,
}

enum BrewerRequest {
    /// Fills, then heats and stirs at once.
    Brew,
    /// Burns, which fails, so the fill after it never starts.
    Scorch,
    /// Starts nothing.
    Ping,
    /// Starts a brew, then rejects the Command.
    BrewThenRefuse,
    CancelLatest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    Fill,
    Heat,
    Stir,
    Burn,
}

/// A step's time is up.
#[derive(Clone)]
struct StepDone {
    job_id: JobId,
    step: Step,
}

struct Brewer;

impl Service for BrewerService {
    type Domain = Brewer;
    type Context = ();
    type Arguments = BrewerArguments;

    const NAME: &'static str = "brewer";
    const VERSION: &'static str = "1.0.0";

    fn build(
        context: &ServiceContext<BrewerArguments>,
    ) -> Result<ServiceBuilder<Brewer>, ServiceError> {
        Ok(ServiceBuilder::new(BrewerSnapshot {
            jobs: Jobs::with_retention(context.arguments().retention),
        })
        .command("Brew", |_: EmptyRequest| Ok(BrewerRequest::Brew))
        .command("Scorch", |_: EmptyRequest| Ok(BrewerRequest::Scorch))
        .command("Ping", |_: EmptyRequest| Ok(BrewerRequest::Ping))
        .command("BrewThenRefuse", |_: EmptyRequest| {
            Ok(BrewerRequest::BrewThenRefuse)
        })
        .command("CancelLatest", |_: EmptyRequest| {
            Ok(BrewerRequest::CancelLatest)
        })
        .jobs())
    }
}

impl Domain for Brewer {
    type Snapshot = BrewerSnapshot;
    type Request = BrewerRequest;
    type IoResult = Infallible;
    type Tick = StepDone;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = Infallible;
    type TimerKey = JobId;

    fn handle(
        snapshot: &mut BrewerSnapshot,
        command: Command<BrewerRequest, Infallible, StepDone, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let started = match command {
            Command::Request(BrewerRequest::Brew) => Ok(snapshot.jobs.start(brew()).leaves),
            Command::Request(BrewerRequest::Scorch) => Ok(snapshot
                .jobs
                .start(JobGraph::Sequence(vec![
                    JobGraph::Leaf(Step::Burn),
                    JobGraph::Leaf(Step::Fill),
                ]))
                .leaves),
            Command::Request(BrewerRequest::Ping) => Ok(Vec::new()),
            Command::Request(BrewerRequest::BrewThenRefuse) => {
                snapshot.jobs.start(brew());
                return Outcome::Rejected {
                    reason: "refused after starting a Job".into(),
                };
            }
            // A cancelling step stops when its time is up, so no leaf starts and no timer changes.
            Command::Request(BrewerRequest::CancelLatest) => snapshot
                .jobs
                .latest_root()
                .map_or(Ok(Vec::new()), |job_id| snapshot.jobs.cancel(job_id))
                .map(|_cancelling| Vec::new()),
            Command::Tick(StepDone { job_id, step }) => {
                let end = match (snapshot.jobs.status(job_id), step) {
                    (Some(JobStatus::Cancelling), _) => JobEnd::Cancelled,
                    (_, Step::Burn) => JobEnd::Failed,
                    (_, Step::Fill | Step::Heat | Step::Stir) => JobEnd::Succeeded,
                };
                snapshot.jobs.finish(job_id, end)
            }
            Command::IoResult(result) => match result {},
            Command::ObservedFact(fact) => match fact {},
        };
        match started {
            Ok(leaves) => Outcome::Applied {
                events: Vec::new(),
                effects: leaves.into_iter().map(schedule).collect(),
            },
            Err(error) => Outcome::Rejected {
                reason: Box::new(error),
            },
        }
    }

    fn io_failed(
        request: Infallible,
        _error: IoError,
    ) -> Command<BrewerRequest, Infallible, StepDone, Infallible> {
        match request {}
    }
}

impl DomainJobs for Brewer {
    type Step = Step;

    fn jobs(snapshot: &BrewerSnapshot) -> &Jobs<Step> {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut BrewerSnapshot) -> &mut Jobs<Step> {
        &mut snapshot.jobs
    }
}

impl Display for Step {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Fill => "fill",
            Self::Heat => "heat",
            Self::Stir => "stir",
            Self::Burn => "burn",
        })
    }
}

#[tokio::test(start_paused = true)]
async fn a_command_that_starts_a_job_acks_its_root_job_id() {
    let harness = start(16).await;

    let first = harness.send("Brew", &EmptyRequest::default()).await;
    let second = harness.send("Brew", &EmptyRequest::default()).await;

    assert!(first.accepted);
    assert_eq!(first.job_id, 1);
    assert_eq!(
        second.job_id, 6,
        "a brew is five Jobs: the root, fill, the parallel, heat and stir"
    );
}

#[tokio::test(start_paused = true)]
async fn a_command_that_starts_no_job_acks_0() {
    let harness = start(16).await;
    harness.send("Brew", &EmptyRequest::default()).await;

    let ping = harness.send("Ping", &EmptyRequest::default()).await;
    let cancel = harness.send("CancelLatest", &EmptyRequest::default()).await;

    assert!(ping.accepted);
    assert_eq!(ping.job_id, JOB_ID_NONE);
    assert!(cancel.accepted);
    assert_eq!(cancel.job_id, JOB_ID_NONE);
}

#[tokio::test(start_paused = true)]
async fn a_rejected_command_rolls_back_the_job_it_started() {
    let harness = start(16).await;

    let refused = harness
        .send("BrewThenRefuse", &EmptyRequest::default())
        .await;

    assert!(!refused.accepted);
    assert_eq!(refused.job_id, JOB_ID_NONE);
    assert_eq!(harness.jobs().await, JobList::default());
    assert_eq!(
        harness.send("Brew", &EmptyRequest::default()).await.job_id,
        1
    );
}

#[tokio::test(start_paused = true)]
async fn the_jobs_state_follows_a_brew_as_its_steps_run() {
    let harness = start(16).await;
    let mut jobs = subscribe(&harness).await;

    harness.send("Brew", &EmptyRequest::default()).await;

    let started = next(&mut jobs).await;
    assert_eq!(
        started
            .jobs
            .iter()
            .map(|job| (job.job_id, job.parent_job_id))
            .collect::<Vec<_>>(),
        [(1, 0), (2, 1), (3, 1), (4, 3), (5, 3)]
    );
    assert_eq!(
        statuses(&started),
        [
            ("sequence", JobStatusStatus::Running),
            ("fill", JobStatusStatus::Running),
            ("parallel", JobStatusStatus::Queued),
            ("heat", JobStatusStatus::Queued),
            ("stir", JobStatusStatus::Queued),
        ]
    );
    assert_eq!(
        statuses(&next(&mut jobs).await),
        [
            ("sequence", JobStatusStatus::Running),
            ("fill", JobStatusStatus::Succeeded),
            ("parallel", JobStatusStatus::Running),
            ("heat", JobStatusStatus::Running),
            ("stir", JobStatusStatus::Running),
        ]
    );
    assert_eq!(
        statuses(&next(&mut jobs).await),
        [
            ("sequence", JobStatusStatus::Running),
            ("fill", JobStatusStatus::Succeeded),
            ("parallel", JobStatusStatus::Running),
            ("heat", JobStatusStatus::Running),
            ("stir", JobStatusStatus::Succeeded),
        ]
    );
    assert_eq!(
        statuses(&next(&mut jobs).await),
        [
            ("sequence", JobStatusStatus::Succeeded),
            ("fill", JobStatusStatus::Succeeded),
            ("parallel", JobStatusStatus::Succeeded),
            ("heat", JobStatusStatus::Succeeded),
            ("stir", JobStatusStatus::Succeeded),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn cancelling_the_root_job_cancels_its_running_leaves() {
    let harness = start(16).await;
    let mut jobs = subscribe(&harness).await;
    harness.send("Brew", &EmptyRequest::default()).await;
    next(&mut jobs).await;

    harness.send("CancelLatest", &EmptyRequest::default()).await;

    assert_eq!(
        statuses(&next(&mut jobs).await),
        [
            ("sequence", JobStatusStatus::Cancelling),
            ("fill", JobStatusStatus::Cancelling),
            ("parallel", JobStatusStatus::Cancelled),
            ("heat", JobStatusStatus::Cancelled),
            ("stir", JobStatusStatus::Cancelled),
        ]
    );
    assert_eq!(
        statuses(&next(&mut jobs).await),
        [
            ("sequence", JobStatusStatus::Cancelled),
            ("fill", JobStatusStatus::Cancelled),
            ("parallel", JobStatusStatus::Cancelled),
            ("heat", JobStatusStatus::Cancelled),
            ("stir", JobStatusStatus::Cancelled),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn finished_root_jobs_beyond_the_retention_count_leave_the_jobs_state() {
    let harness = start(1).await;
    let mut jobs = subscribe(&harness).await;
    harness.send("Scorch", &EmptyRequest::default()).await;
    next(&mut jobs).await;
    assert_eq!(
        statuses(&next(&mut jobs).await),
        [
            ("sequence", JobStatusStatus::Failed),
            ("burn", JobStatusStatus::Failed),
            ("fill", JobStatusStatus::Cancelled),
        ]
    );

    let second = harness.send("Scorch", &EmptyRequest::default()).await;
    next(&mut jobs).await;
    let finished = next(&mut jobs).await;

    assert_eq!(
        finished
            .jobs
            .iter()
            .filter(|job| job.parent_job_id == JOB_ID_NONE)
            .map(|job| job.job_id)
            .collect::<Vec<_>>(),
        [second.job_id]
    );
}

#[tokio::test(start_paused = true)]
async fn info_lists_the_jobs_state() {
    let harness = start(16).await;

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

fn brew() -> JobGraph<Step> {
    JobGraph::Sequence(vec![
        JobGraph::Leaf(Step::Fill),
        JobGraph::Parallel(vec![JobGraph::Leaf(Step::Heat), JobGraph::Leaf(Step::Stir)]),
    ])
}

/// Runs a step as a timer, so heat, the slowest, finishes last.
fn schedule(leaf: LeafJob<Step>) -> Effect<StepDone, Infallible, JobId> {
    let seconds = match leaf.step {
        Step::Fill | Step::Stir | Step::Burn => 1,
        Step::Heat => 2,
    };
    Effect::Schedule {
        after: Duration::from_secs(seconds),
        key: leaf.job_id,
        command: StepDone {
            job_id: leaf.job_id,
            step: leaf.step,
        },
    }
}

async fn start(retention: usize) -> Harness<BrewerService> {
    Harness::start(BrewerArguments { retention }).await.unwrap()
}

async fn subscribe(harness: &Harness<BrewerService>) -> Subscriber {
    harness
        .backend()
        .subscribe(&jobs_key(BrewerService::NAME))
        .await
        .unwrap()
}

/// The next `jobs` State the Service publishes. Time is paused, so it advances to the next step's timer at once.
async fn next(jobs: &mut Subscriber) -> JobList {
    let sample = timeout(Duration::from_secs(10), jobs.recv())
        .await
        .unwrap()
        .unwrap();
    JobList::decode(&sample.payload().to_bytes()).unwrap()
}

fn statuses(jobs: &JobList) -> Vec<(&str, JobStatusStatus)> {
    jobs.jobs
        .iter()
        .map(|job| (job.name.as_str(), job.status))
        .collect()
}
