//! Commands with a body, without a body, and rejected in the Domain.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, SetLevelGoal};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct CommandCookbookService;

#[derive(Clone, Default, clap::Args)]
struct CommandCookbookArguments;

struct CommandCookbook;

#[derive(Clone, Default)]
struct CommandCookbookSnapshot {
    level: u8,
}

enum CommandCookbookRequest {
    SetLevel(u8),
    Reset,
}

impl Service for CommandCookbookService {
    type Domain = CommandCookbook;
    type Context = ();
    type Arguments = CommandCookbookArguments;

    const NAME: &'static str = "cookbook_command";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<CommandCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<CommandCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<CommandCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(CommandCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelGoal| {
                Ok(CommandCookbookRequest::SetLevel(request.level))
            })
            .command("Reset", |_: LevelRequest| Ok(CommandCookbookRequest::Reset)))
    }
}

impl Domain for CommandCookbook {
    type Snapshot = CommandCookbookSnapshot;
    type Request = CommandCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut CommandCookbookSnapshot,
        command: Command<CommandCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(CommandCookbookRequest::SetLevel(level)) => {
                if level > 100 {
                    return Outcome::reject(AboveMaximum {
                        level,
                        maximum: 100,
                    });
                }
                snapshot.level = level;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Request(CommandCookbookRequest::Reset) => {
                snapshot.level = 0;
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

#[derive(Debug, thiserror::Error)]
#[error("{level} is above the maximum of {maximum}")]
struct AboveMaximum {
    level: u8,
    maximum: u8,
}

#[tokio::test(start_paused = true)]
async fn command_with_a_body_updates_state() {
    let harness = Harness::<CommandCookbookService>::start(CommandCookbookArguments)
        .await
        .unwrap();
    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 40 })
        .await
        .unwrap();
    assert!(ack.accepted);
}

#[tokio::test(start_paused = true)]
async fn command_without_a_body_clears_state() {
    let harness = Harness::<CommandCookbookService>::start(CommandCookbookArguments)
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelGoal { level: 40 })
        .await
        .unwrap();
    let ack = harness
        .send("Reset", &LevelRequest::default())
        .await
        .unwrap();
    assert!(ack.accepted);
}

#[tokio::test(start_paused = true)]
async fn domain_rejection_surfaces_as_the_ack_reason() {
    let harness = Harness::<CommandCookbookService>::start(CommandCookbookArguments)
        .await
        .unwrap();
    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 200 })
        .await
        .unwrap();
    assert!(!ack.accepted);
    assert_eq!(ack.reason, "200 is above the maximum of 100");
}
