//! Send a Command from in-process code through [`CommandSender`].

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelResponse, SetLevelGoal};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct CommandSenderCookbookService;

#[derive(Clone, Default, clap::Args)]
struct CommandSenderCookbookArguments;

struct CommandSenderCookbook;

#[derive(Clone, Default)]
struct CommandSenderCookbookSnapshot {
    level: u8,
}

enum CommandSenderCookbookRequest {
    SetLevel(u8),
}

impl Service for CommandSenderCookbookService {
    type Domain = CommandSenderCookbook;
    type Context = ();
    type Arguments = CommandSenderCookbookArguments;

    const NAME: &'static str = "cookbook_command_sender";
    const VERSION: &'static str = "1.0.0";

    fn context(
        _service: &ServiceContext<CommandSenderCookbookArguments>,
    ) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<CommandSenderCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<CommandSenderCookbook>, ServiceError> {
        Ok(
            ServiceBuilder::new(CommandSenderCookbookSnapshot::default())
                .command("SetLevel", |request: SetLevelGoal| {
                    Ok(CommandSenderCookbookRequest::SetLevel(request.level))
                })
                .state("level", |snapshot: &CommandSenderCookbookSnapshot| {
                    LevelResponse {
                        level: snapshot.level,
                        max_level: 100,
                    }
                }),
        )
    }
}

impl Domain for CommandSenderCookbook {
    type Snapshot = CommandSenderCookbookSnapshot;
    type Request = CommandSenderCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut CommandSenderCookbookSnapshot,
        command: Command<CommandSenderCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(CommandSenderCookbookRequest::SetLevel(level)) = command;
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
async fn command_sender_applies_a_request_and_returns_the_ack() {
    let harness = Harness::<CommandSenderCookbookService>::start(CommandSenderCookbookArguments)
        .await
        .unwrap();
    let ack = harness
        .command_sender()
        .send_awaiting_ack(Command::Request(CommandSenderCookbookRequest::SetLevel(9)))
        .await
        .expect("the Inbox accepts the Command");
    assert!(ack.accepted);
    assert_eq!(
        harness.state::<LevelResponse>("level").await.unwrap().level,
        9
    );
}
