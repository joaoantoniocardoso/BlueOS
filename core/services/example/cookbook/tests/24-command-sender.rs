//! Question 24: how do I send a Command from in-process code?
//!
//! The answer is the last test: `command_sender().send_awaiting_ack(...)` puts a typed `Command` into the service's
//! own Inbox and waits for the Domain's verdict (D-27). Shared boilerplate is explained in `01-command.rs`.

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

    // The Command endpoint is declared only so the State can be read back; the proof below does not use it.
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
    // In production a Task or adapter gets this same `CommandSender` from its task context. It skips the wire and
    // the endpoint decoder, so the caller builds the Domain's own Request; `send_awaiting_ack` returns the same
    // ack and rejection an external Command would (D-27).
    let ack = harness
        .command_sender()
        .send_awaiting_ack(Command::Request(CommandSenderCookbookRequest::SetLevel(9)))
        .await
        .expect("the Inbox accepts the Command");
    assert!(ack.accepted);
    // The Snapshot changed, so the Command went through `Domain::handle` like any other.
    assert_eq!(
        harness.state::<LevelResponse>("level").await.unwrap().level,
        9
    );
}
