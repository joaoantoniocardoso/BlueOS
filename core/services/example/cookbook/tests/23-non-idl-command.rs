//! Question 23: how do I accept a non-IDL Command body?
//!
//! The answer is the decoder closure passed to `.command("SetLevel", ...)` in `build`: it turns the wire body into
//! the Domain's own Request and may refuse it with a `Refusal`. Compare `01-command.rs`, where the Domain rejects
//! instead. Shared boilerplate is explained there.
//!
//! The wire body stays an IDL Message, but the Domain's Request is its own type, and parsing happens once at the
//! boundary so the Domain only ever sees valid input (D-26, D-30). That is why `handle` below has a single
//! infallible arm.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::SetLevelGoal;
use blueos_service::{
    Refusal, Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness,
};

struct NonIdlCookbookService;

#[derive(Clone, Default, clap::Args)]
struct NonIdlCookbookArguments;

struct NonIdlCookbook;

#[derive(Clone, Default)]
struct NonIdlCookbookSnapshot {
    level: u8,
}

enum NonIdlCookbookRequest {
    SetLevel(u8),
}

impl Service for NonIdlCookbookService {
    type Domain = NonIdlCookbook;
    type Context = ();
    type Arguments = NonIdlCookbookArguments;

    const NAME: &'static str = "cookbook_non_idl_command";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<NonIdlCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<NonIdlCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<NonIdlCookbook>, ServiceError> {
        Ok(
            ServiceBuilder::new(NonIdlCookbookSnapshot::default()).command(
                "SetLevel",
                |request: SetLevelGoal| {
                    // Returning `Err` here refuses before the Inbox, so the Domain is never called.
                    if request.level > 100 {
                        return Err(Refusal::from(format!(
                            "{} is not a percentage",
                            request.level
                        )));
                    }
                    Ok(NonIdlCookbookRequest::SetLevel(request.level))
                },
            ),
        )
    }
}

impl Domain for NonIdlCookbook {
    type Snapshot = NonIdlCookbookSnapshot;
    type Request = NonIdlCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut NonIdlCookbookSnapshot,
        command: Command<NonIdlCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        // Irrefutable `let`: every other Command type is `Infallible`, and the level was validated by the decoder.
        let Command::Request(NonIdlCookbookRequest::SetLevel(level)) = command;
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
async fn custom_command_validation_rejects_before_the_inbox() {
    let harness = Harness::<NonIdlCookbookService>::start(NonIdlCookbookArguments)
        .await
        .unwrap();
    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 150 })
        .await
        .unwrap();
    // Same ack shape as a Domain rejection, but the reason is the decoder's `Refusal` text.
    assert!(!ack.accepted);
    assert_eq!(ack.reason, "150 is not a percentage");
}

#[tokio::test(start_paused = true)]
async fn custom_command_validation_passes_a_typed_request() {
    let harness = Harness::<NonIdlCookbookService>::start(NonIdlCookbookArguments)
        .await
        .unwrap();
    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 40 })
        .await
        .unwrap();
    assert!(ack.accepted);
}
