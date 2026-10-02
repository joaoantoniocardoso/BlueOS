//! The standard `jobs` State is listed on `info` and readable like any other State.

use core::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
    time::Duration,
};

use blueos_api::{Message, jobs_key};
use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::EmptyRequest,
    blueos_msgs::{JobList, ServiceInfo},
};
use blueos_jobs::{DomainJobs, JobEnd, JobGraph, JobId, Jobs};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct JobStatusCookbookService;

#[derive(Clone, Default, clap::Args)]
struct JobStatusCookbookArguments;

#[derive(Clone)]
struct JobStatusCookbookSnapshot {
    jobs: Jobs<Step>,
}

enum JobStatusCookbookRequest {
    Run,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    Only,
}

#[derive(Clone)]
struct StepDone {
    job_id: JobId,
}

struct JobStatusCookbook;

impl Service for JobStatusCookbookService {
    type Domain = JobStatusCookbook;
    type Context = ();
    type Arguments = JobStatusCookbookArguments;

    const NAME: &'static str = "cookbook_job_status";
    const VERSION: &'static str = "1.0.0";

    fn build(
        _context: &ServiceContext<JobStatusCookbookArguments>,
    ) -> Result<ServiceBuilder<JobStatusCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(JobStatusCookbookSnapshot {
            jobs: Jobs::default(),
        })
        .command("Run", |_: EmptyRequest| Ok(JobStatusCookbookRequest::Run))
        .jobs())
    }
}

impl Domain for JobStatusCookbook {
    type Snapshot = JobStatusCookbookSnapshot;
    type Request = JobStatusCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = StepDone;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = JobId;

    fn handle(
        snapshot: &mut JobStatusCookbookSnapshot,
        command: Command<JobStatusCookbookRequest, Infallible, StepDone, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(JobStatusCookbookRequest::Run) => {
                let started = snapshot.jobs.start(JobGraph::Leaf(Step::Only));
                Outcome::Applied {
                    events: Vec::new(),
                    effects: started
                        .leaves
                        .into_iter()
                        .map(|leaf| Effect::Schedule {
                            after: Duration::from_secs(1),
                            key: leaf.job_id,
                            command: StepDone {
                                job_id: leaf.job_id,
                            },
                        })
                        .collect(),
                }
            }
            Command::Tick(StepDone { job_id }) => {
                snapshot
                    .jobs
                    .finish(job_id, JobEnd::Succeeded)
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

impl DomainJobs for JobStatusCookbook {
    type Step = Step;

    fn jobs(snapshot: &JobStatusCookbookSnapshot) -> &Jobs<Step> {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut JobStatusCookbookSnapshot) -> &mut Jobs<Step> {
        &mut snapshot.jobs
    }
}

impl Display for Step {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("only")
    }
}

#[tokio::test(start_paused = true)]
async fn info_lists_the_jobs_state_and_harness_reads_it() {
    let harness = Harness::<JobStatusCookbookService>::start(JobStatusCookbookArguments)
        .await
        .unwrap();

    let info = harness
        .query::<EmptyRequest, ServiceInfo>("info", &EmptyRequest::default())
        .await
        .expect("the info query answers");
    let jobs_endpoint = info
        .endpoints
        .iter()
        .find(|endpoint| endpoint.name == "jobs")
        .expect("info lists the jobs State");
    assert_eq!(jobs_endpoint.key, jobs_key(JobStatusCookbookService::NAME));
    assert_eq!(jobs_endpoint.response_schema, JobList::SCHEMA_NAME);

    harness.send("Run", &EmptyRequest::default()).await;
    tokio::time::advance(Duration::from_secs(2)).await;

    let jobs = harness.jobs().await;
    assert_eq!(jobs.jobs.len(), 1);
    assert_eq!(jobs.jobs[0].name, "only");
}
