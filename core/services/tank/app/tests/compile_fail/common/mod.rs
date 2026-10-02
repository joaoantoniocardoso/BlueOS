//! A second Domain with the tank's types, for the cases that implement `Conversions` themselves.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, DomainQueries, Now};
use blueos_tank_domain::{Tank, TankEvent, TankQuery, TankRequest, TankResponse, TankSnapshot};

pub struct Twin;

impl Domain for Twin {
    type Snapshot = TankSnapshot;
    type Request = TankRequest;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type Event = TankEvent;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut TankSnapshot,
        _command: Command<TankRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        unimplemented!()
    }
}

impl DomainQueries for Twin {
    type Query = TankQuery;
    type Response = TankResponse;

    fn query(snapshot: &TankSnapshot, query: TankQuery, now: Now) -> TankResponse {
        Tank::query(snapshot, query, now)
    }
}
