//! The pump Domain of `example-minimal` (D-20): set a fill level and read it back. Never shipped.

#![no_std]

extern crate alloc;

use alloc::vec;
use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, DomainQueries, IoError, Now, Outcome};

/// The highest fill level the pump accepts.
pub const MAX_LEVEL: u8 = 100;

/// The teaching example pump.
pub struct Pump;

/// What the pump keeps between Commands.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PumpSnapshot {
    /// Fill level from 0 to [`MAX_LEVEL`].
    pub level: u8,
}

/// Commands a client sends to the pump.
#[derive(Debug, Eq, PartialEq)]
pub enum PumpRequest {
    /// Sets the fill level.
    SetLevel(u8),
}

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

impl Domain for Pump {
    type Snapshot = PumpSnapshot;
    type Request = PumpRequest;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut PumpSnapshot,
        command: Command<PumpRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(PumpRequest::SetLevel(level)) = command;
        if level > MAX_LEVEL {
            return Outcome::reject(AboveMaximum {
                level,
                maximum: MAX_LEVEL,
            });
        }
        snapshot.level = level;
        Outcome::Applied {
            events: vec![],
            effects: vec![],
        }
    }

    fn io_failed(
        request: Infallible,
        _error: IoError,
    ) -> Command<PumpRequest, Infallible, Infallible, Infallible> {
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

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;

    fn now() -> Now {
        Now {
            wall: Duration::from_secs(0),
            monotonic: Duration::from_secs(0),
        }
    }

    #[test]
    fn set_level_updates_the_snapshot_and_level_query() {
        let mut snapshot = PumpSnapshot::default();
        let command = Command::Request(PumpRequest::SetLevel(40));
        let decision = Pump::handle(&mut snapshot, command, now());
        assert!(matches!(decision, Outcome::Applied { .. }));
        assert_eq!(snapshot.level, 40);
        assert_eq!(
            Pump::query(&snapshot, PumpQuery::Level, now()),
            PumpResponse::Level(40)
        );
    }

    #[test]
    fn set_level_rejects_a_level_above_the_maximum() {
        let mut snapshot = PumpSnapshot { level: 40 };
        let command = Command::Request(PumpRequest::SetLevel(101));
        let decision = Pump::handle(&mut snapshot, command, now());
        assert!(matches!(decision, Outcome::Rejected { .. }));
        assert_eq!(snapshot.level, 40);
    }
}
