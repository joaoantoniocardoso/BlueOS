//! Brewer jobs integration-test fixtures.

#![expect(
    unreachable_pub,
    reason = "fixture items are re-exported to integration test roots"
)]
#![expect(
    dead_code,
    reason = "fixture items are shared across sibling integration test binaries"
)]

/// A Job type that runs until its time is up, and that a client may cancel, pause and resume.
pub mod helpers;

use core::{convert::Infallible, time::Duration};
use std::collections::BTreeMap;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::{LevelResponse, SetLevelGoal},
    std_msgs::Empty,
};
use blueos_jobs::{DomainJobs, JobEnd, JobId, JobNature, JobStatus, Jobs};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError};

/// How long a brew takes to pour one cup once it executes.
pub(crate) const BREW_TIME: Duration = Duration::from_secs(60);
/// How many ended Jobs of each type the brewer keeps.
pub const RETENTION: usize = 2;

const BREW: JobNature = JobNature {
    lasting: true,
    cancellable: true,
    pausable: true,
    ..JobNature::INSTANT
};

pub struct BrewerService;

#[derive(Clone)]
pub struct BrewerSnapshot {
    jobs: Jobs,
    /// Every brew that executed, kept after it ends so its Job result can name the cups it poured.
    brews: BTreeMap<JobId, Brew>,
}

#[derive(Clone)]
struct Brew {
    cups: u8,
    poured: u8,
}

pub enum BrewerRequest {
    /// Brews `cups` cups. No cups aborts the Job at once.
    Brew { job_id: JobId, cups: u8 },
    /// Starts nothing.
    Ping,
    /// The Domain rejects it.
    Refuse,
}

pub struct Brewer;

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
        .command("Ping", |_: Empty| Ok(BrewerRequest::Ping))
        .command("Refuse", |_: Empty| Ok(BrewerRequest::Refuse))
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
            helpers::cups(snapshot.brews.get(&job_id).map_or(0, |brew| brew.poured))
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
            Command::Request(BrewerRequest::Brew { job_id, cups: 0 }) => snapshot.jobs.end(
                job_id,
                JobEnd::Aborted {
                    reason: "no cups to brew".to_owned(),
                },
            ),
            Command::Request(BrewerRequest::Brew { job_id, cups }) => {
                snapshot.brews.insert(job_id, Brew { cups, poured: 0 });
                return helpers::pour_next_cup(job_id);
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
                    snapshot.jobs.end(job_id, JobEnd::Canceled)
                } else {
                    let Some(brew) = snapshot.brews.get_mut(&job_id) else {
                        return Outcome::Rejected {
                            reason: "no such brew".into(),
                        };
                    };
                    brew.poured += 1;
                    if brew.poured < brew.cups {
                        return helpers::pour_next_cup(job_id);
                    }
                    snapshot.jobs.end(job_id, JobEnd::Succeeded)
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
