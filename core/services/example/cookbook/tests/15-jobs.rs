//! Run lasting work as a Job: declare its nature, keep its id, and end it when the work finishes.

use core::{convert::Infallible, time::Duration};

use tokio::time::advance;

use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::LevelRequest,
    blueos_msgs::{CommandAckStatus, JobStatusStatus},
};
use blueos_jobs::{DomainJobs, JobControl, JobEnd, JobId, JobNature, JobStatus, Jobs};
use blueos_service::{
    Service, ServiceBuilder, ServiceContext, ServiceError, new_job_id, testing::Harness,
};

/// Brewing runs until its timer fires, and a client may cancel it meanwhile.
const BREW: JobNature = JobNature {
    lasting: true,
    cancellable: true,
    ..JobNature::INSTANT
};

struct JobsCookbookService;

#[derive(Clone, Default, clap::Args)]
struct JobsCookbookArguments;

#[derive(Clone, Default)]
struct JobsCookbookSnapshot {
    jobs: Jobs,
}

enum JobsCookbookRequest {
    Brew { job_id: JobId },
}

#[derive(Clone)]
struct Brewed {
    job_id: JobId,
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
        Ok(ServiceBuilder::new(JobsCookbookSnapshot::default()).job(
            "Brew",
            BREW,
            |job_id, _: LevelRequest| Ok(JobsCookbookRequest::Brew { job_id }),
        ))
    }
}

impl Domain for JobsCookbook {
    type Snapshot = JobsCookbookSnapshot;
    type Request = JobsCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Brewed;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = JobId;

    fn handle(
        snapshot: &mut JobsCookbookSnapshot,
        command: Command<JobsCookbookRequest, Infallible, Brewed, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(JobsCookbookRequest::Brew { job_id }) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Schedule {
                    after: Duration::from_secs(60),
                    key: job_id,
                    command: Brewed { job_id },
                }],
            },
            Command::Tick(Brewed { job_id }) => {
                let canceling = snapshot
                    .jobs
                    .job(job_id)
                    .is_some_and(|job| job.status == JobStatus::Canceling);
                let end = if canceling {
                    JobEnd::Canceled
                } else {
                    JobEnd::Succeeded
                };
                snapshot
                    .jobs
                    .end(job_id, end)
                    .expect("the Brew Job is running");
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
    fn jobs(snapshot: &JobsCookbookSnapshot) -> &Jobs {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut JobsCookbookSnapshot) -> &mut Jobs {
        &mut snapshot.jobs
    }
}

#[tokio::test(start_paused = true)]
async fn a_lasting_job_is_executing_until_its_domain_ends_it() {
    let harness = Harness::<JobsCookbookService>::start(JobsCookbookArguments)
        .await
        .unwrap();
    let job_id = new_job_id();

    let ack = harness
        .submit("Brew", job_id, &LevelRequest::default())
        .await;
    assert!(ack.accepted);
    assert_eq!(ack.job_id, job_id.to_string());
    assert_eq!(ack.status, CommandAckStatus::Executing);

    advance(Duration::from_secs(60)).await;
    let jobs = harness.jobs().await;
    assert_eq!(jobs.jobs[0].status, JobStatusStatus::Succeeded);
}

#[tokio::test(start_paused = true)]
async fn a_cancelled_job_ends_canceled_when_its_work_stops() {
    let harness = Harness::<JobsCookbookService>::start(JobsCookbookArguments)
        .await
        .unwrap();
    let job_id = new_job_id();
    harness
        .submit("Brew", job_id, &LevelRequest::default())
        .await;

    let ack = harness.control(job_id, JobControl::Cancel).await;
    assert!(ack.accepted);
    assert_eq!(ack.status, CommandAckStatus::Canceling);

    advance(Duration::from_secs(60)).await;
    let jobs = harness.jobs().await;
    assert_eq!(jobs.jobs[0].status, JobStatusStatus::Canceled);
}
