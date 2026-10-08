//! Questions 12-13: how do I arm a timer, and cancel a timer?
//!
//! Both are Effects returned from `Domain::handle`: `Effect::Schedule` arms (question 12) and `Effect::Cancel`
//! removes by key (question 13). Shared boilerplate: `01-command.rs`.

use core::{convert::Infallible, time::Duration};

use tokio::time::advance;

use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, LevelResponse};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct TimersCookbookService;

#[derive(Clone, Default, clap::Args)]
struct TimersCookbookArguments;

struct TimersCookbook;

#[derive(Clone, Default)]
struct TimersCookbookSnapshot {
    tick_count: u8,
}

enum TimersCookbookRequest {
    Arm,
    Cancel,
}

// Each Domain declares its own timer key type, never a shared number space (D-03); the key is what `Cancel` names.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TimersCookbookTimerKey {
    Alarm,
}

// The Tick is the Command the Kernel delivers when the timer expires, so a timeout is handled like any other input.
#[derive(Clone)]
enum TimersCookbookTick {
    Fired,
}

impl Service for TimersCookbookService {
    type Domain = TimersCookbook;
    type Context = ();
    type Arguments = TimersCookbookArguments;

    const NAME: &'static str = "cookbook_timers";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TimersCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<TimersCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<TimersCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(TimersCookbookSnapshot::default())
            .command("Arm", |_: LevelRequest| Ok(TimersCookbookRequest::Arm))
            .command("Cancel", |_: LevelRequest| {
                Ok(TimersCookbookRequest::Cancel)
            })
            .state("ticks", |snapshot: &TimersCookbookSnapshot| LevelResponse {
                level: snapshot.tick_count,
                max_level: 0,
            }))
    }
}

impl Domain for TimersCookbook {
    type Snapshot = TimersCookbookSnapshot;
    type Request = TimersCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = TimersCookbookTick;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = TimersCookbookTimerKey;

    fn handle(
        snapshot: &mut TimersCookbookSnapshot,
        command: Command<TimersCookbookRequest, Infallible, TimersCookbookTick, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            // The Domain never sleeps: it asks the Kernel for a timer (D-03), which keeps `handle` testable without a
            // clock.
            Command::Request(TimersCookbookRequest::Arm) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Schedule {
                    after: Duration::from_secs(5),
                    key: TimersCookbookTimerKey::Alarm,
                    command: TimersCookbookTick::Fired,
                }],
            },
            // Schedule then Cancel the same key in one Outcome: the cancel must win over a timer armed alongside it
            // (D-04).
            Command::Request(TimersCookbookRequest::Cancel) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![
                    Effect::Schedule {
                        after: Duration::from_secs(5),
                        key: TimersCookbookTimerKey::Alarm,
                        command: TimersCookbookTick::Fired,
                    },
                    Effect::Cancel(TimersCookbookTimerKey::Alarm),
                ],
            },
            Command::Tick(TimersCookbookTick::Fired) => {
                snapshot.tick_count += 1;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::IoResult(never) => match never {},
            Command::ObservedFact(never) => match never {},
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

#[tokio::test(start_paused = true)]
async fn armed_timer_fires_after_advance() {
    let harness = Harness::<TimersCookbookService>::start(TimersCookbookArguments)
        .await
        .unwrap();
    // `advance` moves the paused clock past the 5 s delay; no real sleeping (D-30).
    harness.send("Arm", &LevelRequest::default()).await.unwrap();
    advance(Duration::from_secs(6)).await;
    assert_eq!(
        harness.state::<LevelResponse>("ticks").await.unwrap().level,
        1
    );
}

#[tokio::test(start_paused = true)]
async fn cancel_prevents_a_rearmed_timer_from_firing() {
    let harness = Harness::<TimersCookbookService>::start(TimersCookbookArguments)
        .await
        .unwrap();
    harness
        .send("Cancel", &LevelRequest::default())
        .await
        .unwrap();
    // Twice the delay: had the timer survived the cancel, the tick count would be 1 by now.
    advance(Duration::from_secs(10)).await;
    assert_eq!(
        harness.state::<LevelResponse>("ticks").await.unwrap().level,
        0
    );
}
