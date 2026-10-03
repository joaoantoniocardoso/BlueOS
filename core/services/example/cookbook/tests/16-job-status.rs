//! The standard `jobs` State is listed on `info` and readable like any other State. Every Command is a Job, so a
//! Service lists its Jobs without tracking them itself.

use core::convert::Infallible;

use blueos_api::{Message, jobs_key};
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::EmptyRequest,
    blueos_msgs::{JobList, JobStatusStatus, ServiceInfo},
};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct JobStatusCookbookService;

#[derive(Clone, Default, clap::Args)]
struct JobStatusCookbookArguments;

enum JobStatusCookbookRequest {
    Run,
}

struct JobStatusCookbook;

impl Service for JobStatusCookbookService {
    type Domain = JobStatusCookbook;
    type Context = ();
    type Arguments = JobStatusCookbookArguments;

    const NAME: &'static str = "cookbook_job_status";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<JobStatusCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<JobStatusCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<JobStatusCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(())
            .command("Run", |_: EmptyRequest| Ok(JobStatusCookbookRequest::Run)))
    }
}

impl Domain for JobStatusCookbook {
    type Snapshot = ();
    type Request = JobStatusCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut (),
        command: Command<JobStatusCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(JobStatusCookbookRequest::Run) => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
            Command::IoResult(never) | Command::Tick(never) | Command::ObservedFact(never) => {
                match never {}
            }
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
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

    let ack = harness.send("Run", &EmptyRequest::default()).await;

    let jobs = harness.jobs().await;
    assert_eq!(jobs.jobs.len(), 1);
    assert_eq!(jobs.jobs[0].job_id, ack.job_id);
    assert_eq!(jobs.jobs[0].job_type, "Run");
    assert_eq!(jobs.jobs[0].status, JobStatusStatus::Succeeded);
}
