//! L1 tests: Domains and Blocks driven by plain function calls, with no runtime.

use core::{
    convert::Infallible,
    error::Error,
    fmt::{self, Display, Formatter},
    time::Duration,
};

use blueos_domain::{Command, Decision, Domain, DomainQueries, Effect, Now, Outcome};

const NOW: Now = Now {
    wall: Duration::from_secs(1_700_000_000),
    monotonic: Duration::from_secs(42),
};

type PumpOutcome = Outcome<PumpEvent, PumpTick, PumpIoRequest, PumpTimerKey>;

/// A Domain with only Requests: no Queries, no Jobs, no IO, no domain events and no timers.
struct Counter;

enum CounterRequest {
    Increment,
}

/// A Domain that composes the Pump Block.
struct Tank;

#[derive(Clone)]
struct TankSnapshot {
    pump: Pump,
}

enum TankRequest {
    StartPump { run_time: Duration },
    StopPump,
}

enum TankQuery {
    PumpRunTime,
}

#[derive(Debug, PartialEq)]
enum TankTick {
    Pump(PumpTick),
}

#[derive(Debug, PartialEq)]
enum TankEvent {
    Pump(PumpEvent),
}

#[derive(Debug, PartialEq)]
enum TankIoRequest {
    Pump(PumpIoRequest),
}

#[derive(Debug, PartialEq)]
enum TankTimerKey {
    Pump(PumpTimerKey),
}

/// A Block: a reusable piece of logic that owns its part of the Snapshot and does not implement `Domain`.
#[derive(Clone)]
enum Pump {
    Idle,
    Running { since: Duration },
}

#[derive(Debug, PartialEq)]
enum PumpEvent {
    Started,
    Stopped { ran_for: Duration },
}

#[derive(Debug, PartialEq)]
enum PumpTick {
    RunTimeElapsed,
}

#[derive(Debug, PartialEq)]
enum PumpIoRequest {
    SetPower { on: bool },
}

#[derive(Debug, PartialEq)]
enum PumpTimerKey {
    RunTime,
}

#[derive(Debug, PartialEq)]
enum PumpRejection {
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
    type TimerKey = Infallible;

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
    fn start(&mut self, run_time: Duration, now: Now) -> PumpOutcome {
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

    fn stop(&mut self, now: Now) -> PumpOutcome {
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

#[test]
fn domain_without_queries_jobs_or_io_handles_a_request() {
    let mut count = 0;

    let decision = Counter::handle(&mut count, Command::Request(CounterRequest::Increment), NOW);

    assert!(
        matches!(decision, Outcome::Applied { events, effects } if events.is_empty() && effects.is_empty())
    );
    assert_eq!(count, 1);
}

#[test]
fn outcome_map_lifts_a_block_into_its_domain() {
    let mut snapshot = TankSnapshot { pump: Pump::Idle };
    let start_pump = TankRequest::StartPump {
        run_time: Duration::from_secs(30),
    };

    let decision = Tank::handle(&mut snapshot, Command::Request(start_pump), NOW);

    let Outcome::Applied { events, effects } = decision else {
        panic!("starting an idle pump must be applied, got {decision:?}");
    };
    assert_eq!(events, vec![TankEvent::Pump(PumpEvent::Started)]);
    assert_eq!(
        effects,
        vec![
            Effect::Io(TankIoRequest::Pump(PumpIoRequest::SetPower { on: true })),
            Effect::Schedule {
                after: Duration::from_secs(30),
                key: TankTimerKey::Pump(PumpTimerKey::RunTime),
                command: TankTick::Pump(PumpTick::RunTimeElapsed),
            },
        ]
    );
}

#[test]
fn rejection_carries_a_typed_reason_through_map() {
    let mut snapshot = TankSnapshot { pump: Pump::Idle };

    let decision = Tank::handle(&mut snapshot, Command::Request(TankRequest::StopPump), NOW);

    let Outcome::Rejected { reason } = decision else {
        panic!("stopping an idle pump must be rejected, got {decision:?}");
    };
    assert_eq!(reason.downcast_ref(), Some(&PumpRejection::NotRunning));
}

#[test]
fn outcome_map_lifts_a_timer_cancel() {
    let mut snapshot = TankSnapshot {
        pump: Pump::Running {
            since: Duration::from_secs(12),
        },
    };

    let decision = Tank::handle(&mut snapshot, Command::Request(TankRequest::StopPump), NOW);

    let Outcome::Applied { events, effects } = decision else {
        panic!("stopping a running pump must be applied, got {decision:?}");
    };
    assert_eq!(
        events,
        vec![TankEvent::Pump(PumpEvent::Stopped {
            ran_for: Duration::from_secs(30),
        })]
    );
    assert_eq!(
        effects,
        vec![
            Effect::Cancel(TankTimerKey::Pump(PumpTimerKey::RunTime)),
            Effect::Io(TankIoRequest::Pump(PumpIoRequest::SetPower { on: false })),
        ]
    );
}

#[test]
fn domain_with_queries_answers_from_the_snapshot() {
    let snapshot = TankSnapshot {
        pump: Pump::Running {
            since: Duration::from_secs(12),
        },
    };

    let response = Tank::query(&snapshot, TankQuery::PumpRunTime, NOW);

    assert_eq!(response, Some(Duration::from_secs(30)));
}
