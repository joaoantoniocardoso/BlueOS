//! A second Domain with the example pump types, for cases that implement `Conversions` themselves.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, DomainQueries, IoError, Now};
use blueos_example_domain::{Pump, PumpQuery, PumpRequest, PumpResponse, PumpSnapshot};
use blueos_jobs::{DomainJobs, JobId, Jobs};

pub struct Twin;

impl Domain for Twin {
    type Snapshot = PumpSnapshot;
    type Request = PumpRequest;
    type IoResult = Infallible;
    type Tick = JobId;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = Infallible;
    type TimerKey = JobId;

    fn handle(
        _snapshot: &mut PumpSnapshot,
        _command: Command<PumpRequest, Infallible, JobId, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        unimplemented!()
    }

    fn io_failed(
        request: Infallible,
        _error: IoError,
    ) -> Command<PumpRequest, Infallible, JobId, Infallible> {
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

impl DomainJobs for Twin {
    fn jobs(snapshot: &PumpSnapshot) -> &Jobs {
        Pump::jobs(snapshot)
    }

    fn jobs_mut(snapshot: &mut PumpSnapshot) -> &mut Jobs {
        Pump::jobs_mut(snapshot)
    }
}
