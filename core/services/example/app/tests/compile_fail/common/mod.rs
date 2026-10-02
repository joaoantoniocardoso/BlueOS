//! A second Domain with the example pump types, for cases that implement `Conversions` themselves.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, DomainQueries, IoError, Now};
use blueos_example_domain::{Pump, PumpQuery, PumpRequest, PumpResponse, PumpSnapshot};

pub struct Twin;

impl Domain for Twin {
    type Snapshot = PumpSnapshot;
    type Request = PumpRequest;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut PumpSnapshot,
        _command: Command<PumpRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        unimplemented!()
    }

    fn io_failed(
        request: Infallible,
        _error: IoError,
    ) -> Command<PumpRequest, Infallible, Infallible, Infallible> {
        match request {}
    }
}

impl DomainQueries for Twin {
    type Query = PumpQuery;
    type Response = PumpResponse;

    fn query(snapshot: &PumpSnapshot, query: PumpQuery, now: Now) -> PumpResponse {
        Pump::query(snapshot, query, now)
    }
}
