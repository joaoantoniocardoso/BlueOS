//! L1 tests: Domains and Blocks driven by plain function calls, with no runtime.

use core::{convert::Infallible, time::Duration};

use blueos_domain::{Command, Decision, Domain, Now, Outcome};

const NOW: Now = Now {
    wall: Duration::from_secs(1_700_000_000),
    monotonic: Duration::from_secs(42),
};

/// A Domain with only Requests: no Queries, no Jobs, no IO, no domain events and no timers.
struct Counter;

enum CounterRequest {
    Increment,
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

#[test]
fn domain_without_queries_jobs_or_io_handles_a_request() {
    let mut count = 0;

    let decision = Counter::handle(&mut count, Command::Request(CounterRequest::Increment), NOW);

    assert!(
        matches!(decision, Outcome::Applied { events, effects } if events.is_empty() && effects.is_empty())
    );
    assert_eq!(count, 1);
}
