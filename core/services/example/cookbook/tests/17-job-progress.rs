//! Partial progress of one root Job shows up in the `jobs` State as each leaf finishes.

use core::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
    time::Duration,
};

use tokio::time::{advance, timeout};

use blueos_api::{Message, jobs_key};
use blueos_comms::Subscriber;
use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::EmptyRequest,
    blueos_msgs::{JobList, JobStatusStatus},
};
use blueos_jobs::{DomainJobs, JobEnd, JobGraph, JobId, JobStatus, Jobs, LeafJob};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct JobProgressCookbookService;

#[derive(Clone, Default, clap::Args)]
struct JobProgressCookbookArguments;

#[derive(Clone)]
struct JobProgressCookbookSnapshot {
    jobs: Jobs<Step>,
}

enum JobProgressCookbookRequest {
    Run,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    Warm,
    Hot,
}

#[derive(Clone)]
struct StepDone {
    job_id: JobId,
    step: Step,
}

struct JobProgressCookbook;

impl Service for JobProgressCookbookService {
    type Domain = JobProgressCookbook;
    type Context = ();
    type Arguments = JobProgressCookbookArguments;

    const NAME: &'static str = "cookbook_job_progress";
    const VERSION: &'static str = "1.0.0";

    fn build(
        _context: &ServiceContext<JobProgressCookbookArguments>,
    ) -> Result<ServiceBuilder<JobProgressCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(JobProgressCookbookSnapshot {
            jobs: Jobs::default(),
        })
        .command("Run", |_: EmptyRequest| Ok(JobProgressCookbookRequest::Run))
        .jobs())
    }
}

impl Domain for JobProgressCookbook {
    type Snapshot = JobProgressCookbookSnapshot;
    type Request = JobProgressCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = StepDone;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = JobId;

    fn handle(
        snapshot: &mut JobProgressCookbookSnapshot,
        command: Command<JobProgressCookbookRequest, Infallible, StepDone, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(JobProgressCookbookRequest::Run) => {
                let started = snapshot.jobs.start(JobGraph::Sequence(vec![
                    JobGraph::Leaf(Step::Warm),
                    JobGraph::Leaf(Step::Hot),
                ]));
                Outcome::Applied {
                    events: Vec::new(),
                    effects: started.leaves.into_iter().map(schedule).collect(),
                }
            }
            Command::Tick(StepDone { job_id, step }) => {
                let end = match (snapshot.jobs.status(job_id), step) {
                    (Some(JobStatus::Cancelling), _) => JobEnd::Cancelled,
                    (_, Step::Warm | Step::Hot) => JobEnd::Succeeded,
                };
                snapshot
                    .jobs
                    .finish(job_id, end)
                    .expect("the leaf is running");
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::IoResult(never) => match never {},
            Command::ObservedFact(never) => match never {},
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

impl DomainJobs for JobProgressCookbook {
    type Step = Step;

    fn jobs(snapshot: &JobProgressCookbookSnapshot) -> &Jobs<Step> {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut JobProgressCookbookSnapshot) -> &mut Jobs<Step> {
        &mut snapshot.jobs
    }
}

impl Display for Step {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Warm => "warm",
            Self::Hot => "hot",
        })
    }
}

fn schedule(leaf: LeafJob<Step>) -> Effect<StepDone, Infallible, JobId> {
    let seconds = match leaf.step {
        Step::Warm => 1,
        Step::Hot => 2,
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

async fn next_jobs_update(subscriber: &mut Subscriber) -> JobList {
    let sample = timeout(Duration::from_secs(10), subscriber.recv())
        .await
        .expect("jobs update arrives before timeout")
        .expect("jobs stream stays open");
    JobList::decode(&sample.payload().to_bytes()).expect("jobs payload decodes")
}

#[tokio::test(start_paused = true)]
async fn jobs_state_shows_the_root_still_running_after_one_leaf_succeeds() {
    let harness = Harness::<JobProgressCookbookService>::start(JobProgressCookbookArguments)
        .await
        .unwrap();
    let mut jobs = harness
        .backend()
        .subscribe(&jobs_key(JobProgressCookbookService::NAME))
        .await
        .unwrap();

    harness.send("Run", &EmptyRequest::default()).await;
    next_jobs_update(&mut jobs).await;

    advance(Duration::from_secs(2)).await;
    let partial = next_jobs_update(&mut jobs).await;

    let root = partial
        .jobs
        .iter()
        .find(|job| job.name == "sequence")
        .expect("the root Job is listed");
    assert_eq!(root.status, JobStatusStatus::Running);
    assert!(
        partial
            .jobs
            .iter()
            .any(|job| { job.name == "warm" && job.status == JobStatusStatus::Succeeded }),
        "the first leaf finished while the root is still running"
    );
    assert!(
        partial
            .jobs
            .iter()
            .any(|job| { job.name == "hot" && job.status == JobStatusStatus::Running }),
        "the second leaf is still running"
    );
}
