//! Question 30: how do I run a Command at startup?
//!
//! The answer is `.on_start(request)` in `build`, and the test proves the ordering the Kernel guarantees: startup
//! Commands enter the Inbox before any endpoint answers, so no client Command runs first (D-04, D-25).

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, PumpState};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct StartupCookbookService;

#[derive(Clone, Default, clap::Args)]
struct StartupCookbookArguments;

struct StartupCookbook;

#[derive(Clone, Default)]
struct StartupCookbookSnapshot {
    steps: u8,
}

enum StartupCookbookRequest {
    Mark,
    Client,
}

impl Service for StartupCookbookService {
    type Domain = StartupCookbook;
    type Context = ();
    type Arguments = StartupCookbookArguments;

    const NAME: &'static str = "cookbook_startup";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<StartupCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<StartupCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<StartupCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(StartupCookbookSnapshot::default())
            // A startup Command is a Request like any other and goes through `handle`; no special init path.
            .on_start(StartupCookbookRequest::Mark)
            .command("Client", |_: LevelRequest| {
                Ok(StartupCookbookRequest::Client)
            })
            .state("progress", |snapshot: &StartupCookbookSnapshot| PumpState {
                level: snapshot.steps,
                max_level: 0,
                self_test_active: false,
                self_test_phase: Default::default(),
            }))
    }
}

impl Domain for StartupCookbook {
    type Snapshot = StartupCookbookSnapshot;
    type Request = StartupCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut StartupCookbookSnapshot,
        command: Command<StartupCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(StartupCookbookRequest::Mark)
            | Command::Request(StartupCookbookRequest::Client) => {
                snapshot.steps += 1;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::IoResult(never) => match never {},
            Command::Tick(never) => match never {},
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
async fn on_start_runs_before_the_first_client_command() {
    let harness = Harness::<StartupCookbookService>::start(StartupCookbookArguments)
        .await
        .unwrap();
    // `Mark` ran at startup and `Client` after it: two steps prove the startup Command was applied exactly once.
    harness
        .send("Client", &LevelRequest::default())
        .await
        .unwrap();
    let progress = harness.state::<PumpState>("progress").await.unwrap();
    assert_eq!(progress.level, 2);
}
