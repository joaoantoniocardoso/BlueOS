//! Questions 1-3: how do I add a Command with a body, a Command with no body, and reject a Command?
//!
//! The answer is the `.command(...)` calls in `build` (declaring the endpoints), the `SetLevel` arm of
//! `Domain::handle` (the rejection), and the three tests at the bottom (what a caller observes).
//!
//! This is the entry that explains the boilerplate every cookbook entry shares; the other entries point here
//! and comment only what they add. Every entry is one `Service` (D-25) over one `Domain` (D-03), driven in-process
//! by `Harness` with the real `context` and `build`.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, SetLevelGoal};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

// The Service type only carries the wiring: its name, its CLI arguments, and the `context`/`build` steps (D-25).
struct CommandCookbookService;

#[derive(Clone, Default, clap::Args)]
struct CommandCookbookArguments;

// The Domain is a marker type: it owns no data, the Snapshot does. Sans-IO and synchronous, so no runtime (D-03).
struct CommandCookbook;

// The Snapshot is the only mutable state `handle` sees, and what a State endpoint would publish.
#[derive(Clone, Default)]
struct CommandCookbookSnapshot {
    level: u8,
}

// A Request is the Domain's own typed form of a wire message; the endpoint decoders below translate into it,
// so the Domain never sees IDL types (D-26).
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

    // `context` is the only step that may do IO (open files, devices). This Service needs nothing, so it is `()`.
    fn context(_service: &ServiceContext<CommandCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    // `build` is pure: it only declares endpoints. A Command is an instant action (D-26); the string is its
    // endpoint name and the closure decodes the wire body into a Request. A Command with no body decodes the
    // empty `LevelRequest` and ignores it.
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
    // The unused associated types are `Infallible`: this Domain emits no Events, does no IO, arms no timers.
    // Each later entry replaces the ones it needs; `match never {}` below proves the arm cannot happen.
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
                // Rejecting is returning `Outcome::reject` with a typed error, with the Snapshot untouched.
                // The Kernel turns its `Display` into the Ack reason (D-26).
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

    // Required by the trait; a Domain with no IoRequest has nothing to map, so the empty `match` is enough.
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

// `start_paused` makes time virtual (D-30): tests never sleep, and timers fire only when the test advances the clock.
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
    // The reason is exactly the typed error's `Display`: the proof that a rejection reaches the caller.
    assert_eq!(ack.reason, "200 is above the maximum of 100");
}
