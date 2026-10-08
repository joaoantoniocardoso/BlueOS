//! Question 17: how do I follow a job through pause, resume and success?
//!
//! The answer is the `HEAT` nature (`pausable`), `next_status` (reading one published `jobs` update), and the test
//! at the bottom, which subscribes to the `jobs` State and drives `control` between updates. The Domain is the
//! `15-jobs.rs` one without cancel handling: pause and resume need no Domain code. Shared boilerplate is explained
//! in `01-command.rs`.

use core::{convert::Infallible, time::Duration};

use tokio::time::{advance, timeout};

use blueos_api::jobs_key;
use blueos_comms::Subscriber;
use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::{
    Message,
    msg::{
        blueos_example_msgs::LevelRequest,
        blueos_msgs::{JobList, JobStatusStatus},
    },
};
use blueos_jobs::{DomainJobs, JobControl, JobEnd, JobId, JobNature, Jobs};
use blueos_service::{
    Service, ServiceBuilder, ServiceContext, ServiceError, new_job_id, testing::Harness,
};

// `pausable` is what makes the Kernel accept Pause and Resume; the Kernel moves the status, so `handle` never sees
// them (D-36).
/// Heating runs until its timer fires, and a client may pause and resume it meanwhile.
const HEAT: JobNature = JobNature {
    lasting: true,
    pausable: true,
    ..JobNature::INSTANT
};

struct JobProgressCookbookService;

#[derive(Clone, Default, clap::Args)]
struct JobProgressCookbookArguments;

#[derive(Clone, Default)]
struct JobProgressCookbookSnapshot {
    jobs: Jobs,
}

enum JobProgressCookbookRequest {
    Heat { job_id: JobId },
}

#[derive(Clone)]
struct Heated {
    job_id: JobId,
}

struct JobProgressCookbook;

impl Service for JobProgressCookbookService {
    type Domain = JobProgressCookbook;
    type Context = ();
    type Arguments = JobProgressCookbookArguments;

    const NAME: &'static str = "cookbook_job_progress";
    const VERSION: &'static str = "1.0.0";

    fn context(
        _service: &ServiceContext<JobProgressCookbookArguments>,
    ) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<JobProgressCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<JobProgressCookbook>, ServiceError> {
        Ok(
            ServiceBuilder::new(JobProgressCookbookSnapshot::default()).job(
                "Heat",
                HEAT,
                |job_id, _: LevelRequest| Ok(JobProgressCookbookRequest::Heat { job_id }),
            ),
        )
    }
}

impl Domain for JobProgressCookbook {
    type Snapshot = JobProgressCookbookSnapshot;
    type Request = JobProgressCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Heated;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = JobId;

    fn handle(
        snapshot: &mut JobProgressCookbookSnapshot,
        command: Command<JobProgressCookbookRequest, Infallible, Heated, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(JobProgressCookbookRequest::Heat { job_id }) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Schedule {
                    after: Duration::from_secs(60),
                    key: job_id,
                    command: Heated { job_id },
                }],
            },
            Command::Tick(Heated { job_id }) => {
                snapshot
                    .jobs
                    .end(job_id, JobEnd::Succeeded)
                    .expect("the Heat Job is running");
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
    fn jobs(snapshot: &JobProgressCookbookSnapshot) -> &Jobs {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut JobProgressCookbookSnapshot) -> &mut Jobs {
        &mut snapshot.jobs
    }
}

// The `jobs` State is published on each status change, so a client follows a Job by subscribing instead of polling
// (D-36).
async fn next_status(subscriber: &mut Subscriber) -> JobStatusStatus {
    let sample = timeout(Duration::from_secs(10), subscriber.recv())
        .await
        .expect("jobs update arrives before timeout")
        .expect("jobs stream stays open");
    let jobs = JobList::decode(&sample.payload().to_bytes()).expect("jobs payload decodes");
    jobs.jobs[0].status
}

#[tokio::test(start_paused = true)]
async fn the_jobs_state_follows_a_job_through_pause_resume_and_success() {
    let harness = Harness::<JobProgressCookbookService>::start(JobProgressCookbookArguments)
        .await
        .unwrap();
    // Subscribing before submitting, with the client-generated id, means no status update can be missed (D-36).
    let mut jobs = harness
        .backend()
        .subscribe(&jobs_key(JobProgressCookbookService::NAME))
        .await
        .unwrap();
    let job_id = new_job_id();

    harness
        .submit("Heat", job_id, &LevelRequest::default())
        .await
        .unwrap();
    // The sequence of updates is the proof: Executing, Paused, Executing, then Succeeded only after the timer.
    assert_eq!(next_status(&mut jobs).await, JobStatusStatus::Executing);
    harness.control(job_id, JobControl::Pause).await.unwrap();
    assert_eq!(next_status(&mut jobs).await, JobStatusStatus::Paused);
    harness.control(job_id, JobControl::Resume).await.unwrap();
    assert_eq!(next_status(&mut jobs).await, JobStatusStatus::Executing);
    advance(Duration::from_secs(60)).await;
    assert_eq!(next_status(&mut jobs).await, JobStatusStatus::Succeeded);
}
