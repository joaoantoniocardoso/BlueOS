//! Start a multi-step Job graph from a Command.

use core::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
    time::Duration,
};

use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::EmptyRequest;
use blueos_jobs::{DomainJobs, JobEnd, JobGraph, JobId, JobStatus, Jobs, LeafJob};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct JobsCookbookService;

#[derive(Clone, Default, clap::Args)]
struct JobsCookbookArguments;

#[derive(Clone)]
struct JobsCookbookSnapshot {
    jobs: Jobs<Step>,
}

enum JobsCookbookRequest {
    Run,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    First,
    Second,
}

#[derive(Clone)]
struct StepDone {
    job_id: JobId,
    step: Step,
}

struct JobsCookbook;

impl Service for JobsCookbookService {
    type Domain = JobsCookbook;
    type Context = ();
    type Arguments = JobsCookbookArguments;

    const NAME: &'static str = "cookbook_jobs";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<JobsCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<JobsCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<JobsCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(JobsCookbookSnapshot {
            jobs: Jobs::default(),
        })
        .command("Run", |_: EmptyRequest| Ok(JobsCookbookRequest::Run))
        .jobs())
    }
}

impl Domain for JobsCookbook {
    type Snapshot = JobsCookbookSnapshot;
    type Request = JobsCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = StepDone;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = JobId;

    fn handle(
        snapshot: &mut JobsCookbookSnapshot,
        command: Command<JobsCookbookRequest, Infallible, StepDone, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(JobsCookbookRequest::Run) => {
                let started = snapshot.jobs.start(JobGraph::Sequence(vec![
                    JobGraph::Leaf(Step::First),
                    JobGraph::Leaf(Step::Second),
                ]));
                Outcome::Applied {
                    events: Vec::new(),
                    effects: started.leaves.into_iter().map(schedule).collect(),
                }
            }
            Command::Tick(StepDone { job_id, step }) => {
                let end = match (snapshot.jobs.status(job_id), step) {
                    (Some(JobStatus::Cancelling), _) => JobEnd::Cancelled,
                    (_, Step::First | Step::Second) => JobEnd::Succeeded,
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

impl DomainJobs for JobsCookbook {
    type Step = Step;

    fn jobs(snapshot: &JobsCookbookSnapshot) -> &Jobs<Step> {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut JobsCookbookSnapshot) -> &mut Jobs<Step> {
        &mut snapshot.jobs
    }
}

impl Display for Step {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::First => "first",
            Self::Second => "second",
        })
    }
}

fn schedule(leaf: LeafJob<Step>) -> Effect<StepDone, Infallible, JobId> {
    Effect::Schedule {
        after: Duration::from_secs(1),
        key: leaf.job_id,
        command: StepDone {
            job_id: leaf.job_id,
            step: leaf.step,
        },
    }
}

#[tokio::test(start_paused = true)]
async fn a_command_that_starts_a_job_graph_acks_the_root_job_id() {
    let harness = Harness::<JobsCookbookService>::start(JobsCookbookArguments)
        .await
        .unwrap();
    let ack = harness.send("Run", &EmptyRequest::default()).await;
    assert!(ack.accepted);
    assert_eq!(ack.job_id, 1);
}
