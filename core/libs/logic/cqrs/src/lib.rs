//! Sans-IO CQRS-lite kernel for BlueOS service domains (D-03).
//!
//! Logic never performs IO, never reads a clock, and never blocks. A command handler updates
//! [`Domain::Snapshot`] and [`Jobs`], then returns a [`Decision`]: domain events plus [`Effect`]s for the
//! runtime kernel to execute. The kernel (D-04) runs IO, arms timers, persists settings, and maps job leaves
//! to IO via [`Domain::io_from_job`].
//!
//! **Time:** If a handler needs "now" or a deadline, the command must carry it (or the kernel injects it when
//! delivering a scheduled command). Handlers must not call into wall-clock APIs.
//!
//! **Replies:** Commands are acknowledged by the kernel (accepted/rejected with a reason). Queries return a
//! [`Domain::View`] from [`App::query`]. Effects do not carry RPC reply bytes.
//!
//! **Publishing:** Adapters derive what to publish from events and snapshot state. Effects do not carry
//! opaque `Vec<u8>` publish payloads.
//!
//! ## Two-step flow with a job and a timeout
//!
//! ```ignore
//! // 1. User starts calibration: handler enqueues a graph and arms a watchdog.
//! jobs.enqueue(JobGraph::Sequence(vec![
//!     JobGraph::Leaf(JobSpec::RequestOffsets),
//!     JobGraph::Leaf(JobSpec::Fit),
//! ]));
//! Decision {
//!     events: vec![Event::Started],
//!     effects: vec![
//!         Effect::Schedule {
//!             after: Duration::from_secs(30),
//!             timer: TimerId(1),
//!             command: Command::StepTimedOut,
//!         },
//!     ],
//! }
//! // App::handle also appends Effect::Io for each runnable leaf after the handler returns.
//!
//! // 2. When offsets arrive, handler cancels the watchdog and completes the job leaf.
//! Decision {
//!     events: vec![Event::OffsetsReceived],
//!     effects: vec![Effect::CancelSchedule(TimerId(1))],
//! }
//! ```
//!
//! On timeout the kernel delivers `Command::StepTimedOut`; the handler cancels the graph and emits failure
//! events without ever having read the clock inside logic.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::time::Duration;

use blueos_jobs::{JobId, Jobs};
use thiserror::Error;

/// Identifier for a kernel-managed timer. Allocated by the domain; the kernel tracks armed timers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TimerId(pub u64);

/// Outcome of handling one command: events to record or publish, and effects for the kernel.
pub struct Decision<D: Domain> {
    pub events: Vec<D::Event>,
    pub effects: Vec<Effect<D::Command, D::IoRequest>>,
}

impl<D: Domain> Default for Decision<D> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }
}

impl<D: Domain> Decision<D> {
    /// Empty decision (no events, no effects).
    pub fn new() -> Self {
        Self::default()
    }
}

/// Side effects the kernel must perform. Logic only describes intent; adapters implement each variant.
#[derive(Clone, Debug, PartialEq)]
pub enum Effect<Command, IoRequest>
where
    Command: Clone,
    IoRequest: Clone,
{
    /// Run blocking or async IO; the adapter posts the result back as a [`Domain::Command`] (for example
    /// job progress or an RPC response mapped by the domain).
    Io(IoRequest),
    /// Arm a one-shot timer. When it fires, the kernel enqueues `command` on the inbox. `after` is relative;
    /// the kernel owns the clock.
    Schedule {
        after: Duration,
        timer: TimerId,
        command: Command,
    },
    /// Disarm a timer previously requested with [`Effect::Schedule`]. Safe if the timer already fired.
    CancelSchedule(TimerId),
    /// Flush domain snapshot and/or settings to durable storage using the adapter's format (D-11).
    Persist,
}

/// Pure domain: command and query handlers plus job-to-IO mapping.
pub trait Domain: Sized {
    type Command: Clone + Send + 'static;
    type Event;
    type Query;
    type View;
    type Snapshot: Clone + Send;
    type IoRequest: Clone + Send;
    type JobSpec: Clone + Send;

    /// Mutate state and describe what the kernel should do next. Must not block or touch the outside world.
    fn handle_command(
        snapshot: &mut Self::Snapshot,
        jobs: &mut Jobs<Self::JobSpec>,
        command: Self::Command,
    ) -> Decision<Self>;

    /// Read-only projection for queries (D-10).
    fn handle_query(
        snapshot: &Self::Snapshot,
        jobs: &Jobs<Self::JobSpec>,
        query: Self::Query,
    ) -> Self::View;

    /// Map a runnable job leaf to the IO request the adapter understands.
    fn io_from_job(job_id: JobId, job_spec: &Self::JobSpec) -> Self::IoRequest;
}

/// Errors surfaced when mutating the job graph through [`App`].
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Jobs(#[from] blueos_jobs::JobsError),
}

/// Holds domain snapshot and job graph; entry point for command and query handling in tests and the kernel.
pub struct App<D: Domain> {
    pub snapshot: D::Snapshot,
    pub jobs: Jobs<D::JobSpec>,
}

impl<D: Domain> Clone for App<D> {
    fn clone(&self) -> Self {
        Self {
            snapshot: self.snapshot.clone(),
            jobs: self.jobs.clone(),
        }
    }
}

impl<D: Domain> App<D> {
    /// New application state with an empty job graph.
    pub fn new(snapshot: D::Snapshot) -> Self {
        Self {
            snapshot,
            jobs: Jobs::new(),
        }
    }

    /// Runs the domain handler, then promotes every newly runnable job leaf to [`Effect::Io`].
    pub fn handle(&mut self, command: D::Command) -> Decision<D> {
        let mut decision = D::handle_command(&mut self.snapshot, &mut self.jobs, command);
        for (job_id, job_spec) in self.jobs.poll_runnable() {
            decision
                .effects
                .push(Effect::Io(D::io_from_job(job_id, &job_spec)));
        }
        decision
    }

    /// Read side: no effects, no mutation.
    pub fn query(&self, query: D::Query) -> D::View {
        D::handle_query(&self.snapshot, &self.jobs, query)
    }

    /// Called by the kernel when an IO job finishes. May make further leaves runnable on the next `handle`.
    pub fn complete_job(&mut self, job_id: JobId, succeeded: bool) -> Result<(), AppError> {
        self.jobs.complete(job_id, succeeded)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use blueos_jobs::{JobGraph, JobStatus};

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum TestJobSpec {
        Work,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum TestCommand {
        Start,
        Timeout,
        CancelTimer,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum TestEvent {
        Started,
        TimedOut,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum TestQuery {
        Phase,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum TestView {
        Idle,
        Running,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct TestSnapshot {
        phase: TestView,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum TestIoRequest {
        DoWork { job_id: JobId },
    }

    struct TestDomain;

    impl Domain for TestDomain {
        type Command = TestCommand;
        type Event = TestEvent;
        type Query = TestQuery;
        type View = TestView;
        type Snapshot = TestSnapshot;
        type IoRequest = TestIoRequest;
        type JobSpec = TestJobSpec;

        fn handle_command(
            snapshot: &mut Self::Snapshot,
            jobs: &mut Jobs<Self::JobSpec>,
            command: Self::Command,
        ) -> Decision<Self> {
            match command {
                TestCommand::Start => {
                    jobs.enqueue(JobGraph::Leaf(TestJobSpec::Work));
                    snapshot.phase = TestView::Running;
                    Decision {
                        events: vec![TestEvent::Started],
                        effects: vec![Effect::Schedule {
                            after: Duration::from_secs(5),
                            timer: TimerId(1),
                            command: TestCommand::Timeout,
                        }],
                    }
                }
                TestCommand::Timeout => {
                    snapshot.phase = TestView::Idle;
                    Decision {
                        events: vec![TestEvent::TimedOut],
                        effects: Vec::new(),
                    }
                }
                TestCommand::CancelTimer => Decision {
                    events: Vec::new(),
                    effects: vec![Effect::CancelSchedule(TimerId(1))],
                },
            }
        }

        fn handle_query(
            snapshot: &Self::Snapshot,
            _jobs: &Jobs<Self::JobSpec>,
            _query: Self::Query,
        ) -> Self::View {
            snapshot.phase.clone()
        }

        fn io_from_job(job_id: JobId, _job_spec: &Self::JobSpec) -> Self::IoRequest {
            TestIoRequest::DoWork { job_id }
        }
    }

    #[test]
    fn start_enqueues_io_and_schedule() {
        let mut app = App::<TestDomain>::new(TestSnapshot {
            phase: TestView::Idle,
        });
        let decision = app.handle(TestCommand::Start);
        assert_eq!(decision.events, vec![TestEvent::Started]);
        assert_eq!(decision.effects.len(), 2);
        assert!(decision.effects.iter().any(|effect| {
            matches!(
                effect,
                Effect::Schedule {
                    after,
                    timer: TimerId(1),
                    command: TestCommand::Timeout,
                } if *after == Duration::from_secs(5)
            )
        }));
        assert!(decision.effects.iter().any(|effect| {
            matches!(effect, Effect::Io(TestIoRequest::DoWork { job_id }) if *job_id == JobId(0))
        }));
        assert_eq!(app.query(TestQuery::Phase), TestView::Running);
    }

    #[test]
    fn cancel_schedule_passes_through_handle() {
        let mut app = App::<TestDomain>::new(TestSnapshot {
            phase: TestView::Idle,
        });
        let decision = app.handle(TestCommand::CancelTimer);
        assert!(decision.events.is_empty());
        assert_eq!(decision.effects, vec![Effect::CancelSchedule(TimerId(1))]);
    }

    #[test]
    fn timeout_command_updates_snapshot() {
        let mut app = App::<TestDomain>::new(TestSnapshot {
            phase: TestView::Running,
        });
        let decision = app.handle(TestCommand::Timeout);
        assert_eq!(decision.events, vec![TestEvent::TimedOut]);
        assert!(decision.effects.is_empty());
        assert_eq!(app.query(TestQuery::Phase), TestView::Idle);
    }

    #[test]
    fn complete_job_after_start() {
        let mut app = App::<TestDomain>::new(TestSnapshot {
            phase: TestView::Idle,
        });
        app.handle(TestCommand::Start);
        app.complete_job(JobId(0), true).unwrap();
        assert_eq!(app.jobs.status(JobId(0)), Some(JobStatus::Succeeded));
    }
}
