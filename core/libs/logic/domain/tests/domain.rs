//! L1 tests: Domains and Blocks driven by plain function calls, with no runtime.

mod common;

use core::time::Duration;

use blueos_domain::{Command, Domain, DomainQueries, Effect, IoError, Outcome};

use common::{
    Counter, CounterRequest, NOW, Pump, PumpEvent, PumpIoRequest, PumpRejection, PumpTick, Tank,
    TankEvent, TankIoRequest, TankQuery, TankRequest, TankSnapshot, TankTick, TankTimerKey,
};

#[test]
fn io_error_carries_a_message_for_the_domain() {
    let error = IoError::new("disk full");

    assert_eq!(error.message(), "disk full");
    assert_eq!(error.to_string(), "disk full");
}

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
                key: TankTimerKey::Pump(common::PumpTimerKey::RunTime),
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
            Effect::Cancel(TankTimerKey::Pump(common::PumpTimerKey::RunTime)),
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
