//! Question 16: how do I show job status in the UI?
//!
//! The answer is that there is nothing to write: the Service declares only a Command, and the Kernel publishes the
//! `jobs` State for it (D-36, D-12). The test shows what a UI reads: the `jobs` endpoint on `info`, then the
//! `JobList` through `Harness::jobs`. This Domain keeps no Jobs, unlike `15-jobs.rs`, because an instant Command
//! ends in the step that accepts it. Shared boilerplate is explained in `01-command.rs`.

use core::convert::Infallible;

use blueos_api::{Message, jobs_key};
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::LevelRequest,
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
            .command("Run", |_: LevelRequest| Ok(JobStatusCookbookRequest::Run)))
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

    // A UI discovers the State from `info` (D-12) rather than hard-coding its key.
    let info = harness
        .query::<LevelRequest, ServiceInfo>("info", &LevelRequest::default())
        .await
        .expect("the harness reaches the info query")
        .expect("the info query answers");
    let jobs_endpoint = info
        .endpoints
        .iter()
        .find(|endpoint| endpoint.name == "jobs")
        .expect("info lists the jobs State");
    assert_eq!(jobs_endpoint.key, jobs_key(JobStatusCookbookService::NAME));
    assert_eq!(jobs_endpoint.interface_type, JobList::SCHEMA_NAME);

    let ack = harness.send("Run", &LevelRequest::default()).await.unwrap();

    // One entry, already Succeeded and keyed by the ack's id, proves the Command was tracked as a Job although
    // the Service never mentions Jobs.
    let jobs = harness.jobs().await.unwrap();
    assert_eq!(jobs.jobs.len(), 1);
    assert_eq!(jobs.jobs[0].job_id, ack.job_id);
    assert_eq!(jobs.jobs[0].job_type, "Run");
    assert_eq!(jobs.jobs[0].status, JobStatusStatus::Succeeded);
}
