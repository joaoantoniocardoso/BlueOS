//! Question 5: how do I publish a State?
//!
//! The answer is the `.state(...)` declaration in `build`: a pure selector from the Snapshot to a Message. The
//! Domain never publishes; the Kernel does after each applied Command. Shared boilerplate: `01-command.rs`.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{PumpState, SetLevelGoal};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct StateCookbookService;

#[derive(Clone, Default, clap::Args)]
struct StateCookbookArguments;

struct StateCookbook;

#[derive(Clone, Default)]
struct StateCookbookSnapshot {
    level: u8,
}

enum StateCookbookRequest {
    SetLevel(u8),
}

impl Service for StateCookbookService {
    type Domain = StateCookbook;
    type Context = ();
    type Arguments = StateCookbookArguments;

    const NAME: &'static str = "cookbook_state";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<StateCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<StateCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<StateCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(StateCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelGoal| {
                Ok(StateCookbookRequest::SetLevel(request.level))
            })
            // A State is a selector from the Snapshot to a Message (D-26). It has no side effects (D-27): the Kernel
            // re-runs it after every applied Command and publishes only when the value changed.
            .state("pump", |snapshot: &StateCookbookSnapshot| PumpState {
                level: snapshot.level,
                max_level: 100,
                self_test_active: false,
                self_test_phase: Default::default(),
            }))
    }
}

impl Domain for StateCookbook {
    type Snapshot = StateCookbookSnapshot;
    type Request = StateCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut StateCookbookSnapshot,
        command: Command<StateCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(StateCookbookRequest::SetLevel(level)) = command;
        snapshot.level = level;
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
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
async fn state_publishes_after_a_command() {
    let harness = Harness::<StateCookbookService>::start(StateCookbookArguments)
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelGoal { level: 77 })
        .await
        .unwrap();
    // `harness.state` reads what a subscriber would see, so this proves the Snapshot change reached the bus.
    let pump = harness.state::<PumpState>("pump").await.unwrap();
    assert_eq!(pump.level, 77);
}
