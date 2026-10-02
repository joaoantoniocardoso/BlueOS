//! The Domain of the tank, the test Service of the generated endpoint registration (D-26): a client fills the tank
//! to a level, drains it and reads the level back. It is never shipped.

#![no_std]

extern crate alloc;

use alloc::vec;
use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, DomainQueries, Now, Outcome};

/// The tank.
pub struct Tank;

/// What the tank keeps between Commands.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TankSnapshot {
    /// How full the tank is.
    pub level: Percent,
}

/// The Commands a client sends to the tank.
#[derive(Debug, Eq, PartialEq)]
pub enum TankRequest {
    /// Fills or empties the tank to a level.
    SetLevel(Percent),
    /// Empties the tank.
    Drain,
}

/// What happened to the tank.
#[derive(Debug, Eq, PartialEq)]
pub enum TankEvent {
    /// The tank is now at this level.
    LevelChanged(Percent),
}

/// The questions a client asks the tank.
#[derive(Debug, Eq, PartialEq)]
pub enum TankQuery {
    /// How full the tank is.
    Level,
    /// How full the tank would be after adding this much, at most full.
    LevelAfterFill(Percent),
}

/// The tank's answers.
#[derive(Debug, Eq, PartialEq)]
pub enum TankResponse {
    /// A level.
    Level(Percent),
}

/// A fill level, from 0 to 100 percent.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Percent(u8);

/// A number above 100, which is not a [`Percent`].
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
#[error("{0} is not a percentage")]
pub struct NotAPercent(pub u8);

impl Domain for Tank {
    type Snapshot = TankSnapshot;
    type Request = TankRequest;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type Event = TankEvent;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut TankSnapshot,
        command: Command<TankRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(request) = command;
        snapshot.level = match request {
            TankRequest::SetLevel(level) => level,
            TankRequest::Drain => Percent::default(),
        };
        Outcome::Applied {
            events: vec![TankEvent::LevelChanged(snapshot.level)],
            effects: vec![],
        }
    }
}

impl DomainQueries for Tank {
    type Query = TankQuery;
    type Response = TankResponse;

    fn query(snapshot: &TankSnapshot, query: TankQuery, _now: Now) -> TankResponse {
        match query {
            TankQuery::Level => TankResponse::Level(snapshot.level),
            TankQuery::LevelAfterFill(added) => {
                TankResponse::Level(Percent(snapshot.level.0.saturating_add(added.0).min(100)))
            }
        }
    }
}

impl Percent {
    /// The level `value` percent.
    ///
    /// # Errors
    ///
    /// [`NotAPercent`] when `value` is above 100.
    pub const fn new(value: u8) -> Result<Self, NotAPercent> {
        if value > 100 {
            Err(NotAPercent(value))
        } else {
            Ok(Self(value))
        }
    }

    /// The level as a number from 0 to 100.
    pub const fn get(self) -> u8 {
        self.0
    }
}
