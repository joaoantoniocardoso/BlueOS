//! The pump Domain of `example-minimal`: fill to a level one step at a time, and read it back. Never shipped.

#![no_std]

extern crate alloc;

use alloc::vec;
use core::{convert::Infallible, time::Duration};

use blueos_domain::{Command, Decision, Domain, DomainQueries, Effect, IoError, Now, Outcome};
use blueos_jobs::{DomainJobs, JobEnd, JobId, Jobs};

/// The highest fill level the pump accepts.
pub const MAX_LEVEL: u8 = 100;

/// How long the pump takes to move the level by one.
pub const STEP: Duration = Duration::from_secs(1);

/// The teaching example pump.
pub struct Pump;

/// What the pump keeps between Commands.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PumpSnapshot {
    /// Fill level from 0 to [`MAX_LEVEL`].
    pub level: u8,
    /// The `SetLevel` Job the pump is filling for, if any.
    pub filling: Option<Filling>,
    /// The Jobs of the pump.
    pub jobs: Jobs,
}

/// A `SetLevel` Job the pump is filling for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Filling {
    /// The Job.
    pub job_id: JobId,
    /// The level it reaches.
    pub target: Level,
}

/// Commands a client sends to the pump.
#[derive(Debug, Eq, PartialEq)]
pub enum PumpRequest {
    /// Fills or drains to a level, as the Job `job_id`.
    SetLevel {
        /// The Job.
        job_id: JobId,
        /// The level to reach.
        level: Level,
    },
}

/// A fill level the pump accepts: at most [`MAX_LEVEL`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Level(u8);

/// Questions a client asks the pump.
#[derive(Debug, Eq, PartialEq)]
pub enum PumpQuery {
    /// The current fill level.
    Level,
}

/// Answers the pump returns.
#[derive(Debug, Eq, PartialEq)]
pub enum PumpResponse {
    /// A fill level.
    Level(u8),
}

/// A level above [`MAX_LEVEL`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
#[error("{level} is above the maximum level of {maximum}")]
pub struct AboveMaximum {
    /// The requested level.
    pub level: u8,
    /// The maximum the pump accepts.
    pub maximum: u8,
}

/// A `SetLevel` while the pump fills for another one.
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
#[error("the pump is filling for the Job {job_id}")]
pub struct Busy {
    /// The Job the pump fills for.
    pub job_id: JobId,
}

impl Domain for Pump {
    type Snapshot = PumpSnapshot;
    type Request = PumpRequest;
    type IoResult = Infallible;
    /// A step of the `SetLevel` Job it names.
    type Tick = JobId;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = Infallible;
    type TimerKey = JobId;

    fn handle(
        snapshot: &mut PumpSnapshot,
        command: Command<PumpRequest, Infallible, JobId, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let filling = match command {
            Command::Request(PumpRequest::SetLevel { job_id, level }) => {
                if let Some(filling) = snapshot.filling {
                    return Outcome::reject(Busy {
                        job_id: filling.job_id,
                    });
                }
                Filling {
                    job_id,
                    target: level,
                }
            }
            Command::Tick(job_id) => {
                let Some(filling) = snapshot.filling.filter(|filling| filling.job_id == job_id)
                else {
                    return Outcome::Applied {
                        events: vec![],
                        effects: vec![],
                    };
                };
                let target = filling.target.get();
                snapshot.level = if snapshot.level < target {
                    snapshot.level + 1
                } else {
                    snapshot.level - 1
                };
                filling
            }
            Command::IoResult(never) => match never {},
            Command::ObservedFact(never) => match never {},
        };
        if snapshot.level != filling.target.get() {
            snapshot.filling = Some(filling);
            return Outcome::Applied {
                events: vec![],
                effects: vec![Effect::Schedule {
                    after: STEP,
                    key: filling.job_id,
                    command: filling.job_id,
                }],
            };
        }
        snapshot.filling = None;
        if let Err(error) = snapshot.jobs.end(filling.job_id, JobEnd::Succeeded, "") {
            return Outcome::reject(error);
        }
        Outcome::Applied {
            events: vec![],
            effects: vec![],
        }
    }

    fn io_failed(
        request: Infallible,
        _error: IoError,
    ) -> Command<PumpRequest, Infallible, JobId, Infallible> {
        match request {}
    }
}

impl DomainQueries for Pump {
    type Query = PumpQuery;
    type Response = PumpResponse;

    fn query(snapshot: &PumpSnapshot, query: PumpQuery, _now: Now) -> PumpResponse {
        match query {
            PumpQuery::Level => PumpResponse::Level(snapshot.level),
        }
    }
}

impl DomainJobs for Pump {
    fn jobs(snapshot: &PumpSnapshot) -> &Jobs {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut PumpSnapshot) -> &mut Jobs {
        &mut snapshot.jobs
    }
}

impl Level {
    /// `level`, or why the pump does not accept it.
    pub const fn new(level: u8) -> Result<Self, AboveMaximum> {
        if level > MAX_LEVEL {
            Err(AboveMaximum {
                level,
                maximum: MAX_LEVEL,
            })
        } else {
            Ok(Self(level))
        }
    }

    /// The level as a number.
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use blueos_jobs::{JobNature, JobStatus};

    use super::*;

    const SET_LEVEL: JobNature = JobNature {
        lasting: true,
        ..JobNature::INSTANT
    };

    fn now() -> Now {
        Now {
            wall: Duration::from_secs(0),
            monotonic: Duration::from_secs(0),
        }
    }

    /// The snapshot after submitting the Job `job_id` for `level`, as the Kernel does before the Domain handles it.
    fn submit(snapshot: &mut PumpSnapshot, job_id: JobId, level: u8) -> Decision<Pump> {
        snapshot
            .jobs
            .submit(job_id, "SetLevel", &[], SET_LEVEL)
            .unwrap();
        let level = Level::new(level).unwrap();
        Pump::handle(
            snapshot,
            Command::Request(PumpRequest::SetLevel { job_id, level }),
            now(),
        )
    }

    #[test]
    fn set_level_moves_one_step_per_tick_and_succeeds_at_the_level() {
        let mut snapshot = PumpSnapshot::default();
        let job_id = JobId::from_u128(1);

        assert!(matches!(
            submit(&mut snapshot, job_id, 2),
            Outcome::Applied { .. }
        ));
        assert_eq!(snapshot.level, 0);
        Pump::handle(&mut snapshot, Command::Tick(job_id), now());
        assert_eq!(snapshot.level, 1);
        assert_eq!(
            snapshot.jobs.job(job_id).unwrap().status,
            JobStatus::Executing
        );
        Pump::handle(&mut snapshot, Command::Tick(job_id), now());

        assert_eq!(snapshot.level, 2);
        assert_eq!(snapshot.filling, None);
        assert_eq!(
            snapshot.jobs.job(job_id).unwrap().status,
            JobStatus::Succeeded
        );
        assert_eq!(
            Pump::query(&snapshot, PumpQuery::Level, now()),
            PumpResponse::Level(2)
        );
    }

    #[test]
    fn set_level_rejects_a_second_job_while_filling() {
        let mut snapshot = PumpSnapshot::default();
        let [filling, second] = [1, 2].map(JobId::from_u128);
        submit(&mut snapshot, filling, 3);

        let decision = submit(&mut snapshot, second, 1);

        assert!(matches!(decision, Outcome::Rejected { .. }));
    }

    #[test]
    fn a_level_above_the_maximum_is_not_a_level() {
        assert_eq!(
            Level::new(MAX_LEVEL + 1),
            Err(AboveMaximum {
                level: MAX_LEVEL + 1,
                maximum: MAX_LEVEL,
            })
        );
        assert_eq!(Level::new(MAX_LEVEL).map(Level::get), Ok(MAX_LEVEL));
    }
}
