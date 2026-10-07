//! Shared Counter and Tank fixtures for L1 Domain and Block tests.

use core::{
    convert::Infallible,
    error::Error,
    fmt::{self, Display, Formatter},
    time::Duration,
};

use blueos_domain::{Command, Decision, Domain, DomainQueries, Effect, IoError, Now, Outcome};

pub(crate) const NOW: Now = Now {
    wall: Duration::from_secs(1_700_000_000),
    monotonic: Duration::from_secs(42),
};

pub(crate) type PumpOutcome = Outcome<PumpEvent, PumpTick, PumpIoRequest, PumpTimerKey>;

/// A Domain with only Requests: no Queries, no Jobs, no IO, no domain events and no timers.
pub(crate) struct Counter;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CounterTimerKey {}

pub(crate) enum CounterRequest {
    Increment,
}

/// A Domain that composes the Pump Block.
pub(crate) struct Tank;

#[derive(Clone)]
pub(crate) struct TankSnapshot {
    pub pump: Pump,
}

pub(crate) enum TankRequest {
    StartPump { run_time: Duration },
    StopPump,
}

pub(crate) enum TankQuery {
    PumpRunTime,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TankTick {
    Pump(PumpTick),
}

#[derive(Debug, PartialEq)]
pub(crate) enum TankEvent {
    Pump(PumpEvent),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TankIoRequest {
    Pump(PumpIoRequest),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum TankTimerKey {
    Pump(PumpTimerKey),
}

/// A Block: a reusable piece of logic that owns its part of the Snapshot and does not implement `Domain`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Pump {
    Idle,
    Running { since: Duration },
}

#[derive(Debug, PartialEq)]
pub(crate) enum PumpEvent {
    Started,
    Stopped { ran_for: Duration },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PumpTick {
    RunTimeElapsed,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PumpIoRequest {
    SetPower { on: bool },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PumpTimerKey {
    RunTime,
}

#[derive(Debug, PartialEq)]
pub(crate) enum PumpRejection {
    NotRunning,
}

impl Domain for Counter {
    type Snapshot = u32;
    type Request = CounterRequest;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = Infallible;
    type TimerKey = CounterTimerKey;

    fn handle(
        snapshot: &mut Self::Snapshot,
        command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(CounterRequest::Increment) => *snapshot += 1,
        }
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

impl Domain for Tank {
    type Snapshot = TankSnapshot;
    type Request = TankRequest;
    type IoResult = Infallible;
    type Tick = TankTick;
    type ObservedFact = Infallible;
    type Event = TankEvent;
    type IoRequest = TankIoRequest;
    type TimerKey = TankTimerKey;

    fn handle(
        snapshot: &mut Self::Snapshot,
        command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(TankRequest::StartPump { run_time }) => {
                snapshot.pump.start(run_time, now).map(
                    TankEvent::Pump,
                    TankTick::Pump,
                    TankIoRequest::Pump,
                    TankTimerKey::Pump,
                )
            }
            Command::Request(TankRequest::StopPump) => snapshot.pump.stop(now).map(
                TankEvent::Pump,
                TankTick::Pump,
                TankIoRequest::Pump,
                TankTimerKey::Pump,
            ),
            Command::Tick(TankTick::Pump(PumpTick::RunTimeElapsed)) => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
            Command::IoResult(io_result) => match io_result {},
            Command::ObservedFact(observed_fact) => match observed_fact {},
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {
            TankIoRequest::Pump(PumpIoRequest::SetPower { on: _ }) => {
                panic!("the Tank tests do not run IO through the Kernel");
            }
        }
    }
}

impl DomainQueries for Tank {
    type Query = TankQuery;
    type Response = Option<Duration>;

    fn query(snapshot: &Self::Snapshot, query: Self::Query, now: Now) -> Self::Response {
        match (query, &snapshot.pump) {
            (TankQuery::PumpRunTime, Pump::Running { since }) => Some(now.monotonic - *since),
            (TankQuery::PumpRunTime, Pump::Idle) => None,
        }
    }
}

impl Pump {
    pub(crate) fn start(&mut self, run_time: Duration, now: Now) -> PumpOutcome {
        *self = Self::Running {
            since: now.monotonic,
        };
        Outcome::Applied {
            events: vec![PumpEvent::Started],
            effects: vec![
                Effect::Io(PumpIoRequest::SetPower { on: true }),
                Effect::Schedule {
                    after: run_time,
                    key: PumpTimerKey::RunTime,
                    command: PumpTick::RunTimeElapsed,
                },
            ],
        }
    }

    pub(crate) fn stop(&mut self, now: Now) -> PumpOutcome {
        let Self::Running { since } = *self else {
            return Outcome::reject(PumpRejection::NotRunning);
        };
        *self = Self::Idle;
        Outcome::Applied {
            events: vec![PumpEvent::Stopped {
                ran_for: now.monotonic - since,
            }],
            effects: vec![
                Effect::Cancel(PumpTimerKey::RunTime),
                Effect::Io(PumpIoRequest::SetPower { on: false }),
            ],
        }
    }
}

impl Display for PumpRejection {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRunning => formatter.write_str("the pump is not running"),
        }
    }
}

impl Error for PumpRejection {}
