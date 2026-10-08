//! Question 15: how do I run lasting work as a Job, and cancel it?
//!
//! The answer is the `BREW` nature and the `.job(...)` call in `build` (declaring the Job), the `Brew` and `Tick`
//! arms of `Domain::handle` (keeping the id and ending the Job), the `DomainJobs` impl (where the Snapshot keeps
//! its Jobs), and the two tests at the bottom (run to success, cancel). Shared boilerplate is explained in
//! `01-command.rs`.
//!
//! Every client Request is a Job (D-36): an instant Command is the same mechanism with `JobNature::INSTANT`.

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

// The nature is declared in code so `ServiceInfo` publishes it and a UI offers only the actions that apply, such
// as Cancel, before sending anything (D-36). Anything not set here stays as `INSTANT` leaves it.
/// Brewing runs until its timer fires, and a client may cancel it meanwhile.
const BREW: JobNature = JobNature {
    lasting: true,
    cancellable: true,
    ..JobNature::INSTANT
};

struct JobsCookbookService;

#[derive(Clone, Default, clap::Args)]
struct JobsCookbookArguments;

// The Snapshot holds the Jobs, exposed to the Kernel by `DomainJobs` below: it is the source of truth for the
// `jobs` State and for control Requests.
#[derive(Clone, Default)]
struct JobsCookbookSnapshot {
    jobs: Jobs,
}

// The client generates the Job id and the endpoint decoder passes it in, so the Domain can key its timer by it (D-36).
enum JobsCookbookRequest {
    Brew { job_id: JobId },
}

// The Tick is the timer's payload: it carries the id so the Domain knows which Job's work finished.
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
        // `.job` replaces `.command` for lasting work: it takes the nature and hands the decoder the client's Job id.
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
            // The Job stays Executing because the Domain does not end it here; the timer (keyed by the id, so a
            // cancel could revoke it) later returns the end as a Tick.
            Command::Request(JobsCookbookRequest::Brew { job_id }) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Schedule {
                    after: Duration::from_secs(60),
                    key: job_id,
                    command: Brewed { job_id },
                }],
            },
            Command::Tick(Brewed { job_id }) => {
                // A cancel only moves the Job to Canceling; the Job ends Canceled once its work stops, which here
                // is this Tick. Cooperative cancellation is the Domain's choice (D-36).
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
        .await
        .unwrap();
    assert!(ack.accepted);
    assert_eq!(ack.job_id, job_id.to_string());
    // A lasting Job acks as Executing instead of a final status; only an instant one takes the fast path (D-36).
    assert_eq!(ack.status, CommandAckStatus::Executing);

    advance(Duration::from_secs(60)).await;
    let jobs = harness.jobs().await.unwrap();
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
        .await
        .unwrap();

    let ack = harness.control(job_id, JobControl::Cancel).await.unwrap();
    assert!(ack.accepted);
    // Control acts at once and never queues (D-36), so the ack is Canceling; the final Canceled follows the Tick.
    assert_eq!(ack.status, CommandAckStatus::Canceling);

    advance(Duration::from_secs(60)).await;
    let jobs = harness.jobs().await.unwrap();
    assert_eq!(jobs.jobs[0].status, JobStatusStatus::Canceled);
}
