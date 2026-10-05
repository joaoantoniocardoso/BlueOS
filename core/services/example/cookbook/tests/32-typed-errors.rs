//! Typed errors become client-visible reasons without string matching.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::SetLevelGoal;
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct TypedErrorsCookbookService;

#[derive(Clone, Default, clap::Args)]
struct TypedErrorsCookbookArguments;

struct TypedErrorsCookbook;

#[derive(Clone, Default)]
struct TypedErrorsCookbookSnapshot {
    level: u8,
}

enum TypedErrorsCookbookRequest {
    SetLevel(u8),
}

#[derive(Debug, thiserror::Error)]
#[error("level {level} is not allowed")]
struct DisallowedLevel {
    level: u8,
}

impl Service for TypedErrorsCookbookService {
    type Domain = TypedErrorsCookbook;
    type Context = ();
    type Arguments = TypedErrorsCookbookArguments;

    const NAME: &'static str = "cookbook_typed_errors";
    const VERSION: &'static str = "1.0.0";

    fn context(
        _service: &ServiceContext<TypedErrorsCookbookArguments>,
    ) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<TypedErrorsCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<TypedErrorsCookbook>, ServiceError> {
        Ok(
            ServiceBuilder::new(TypedErrorsCookbookSnapshot::default()).command(
                "SetLevel",
                |request: SetLevelGoal| {
                    if request.level == 13 {
                        return Err(Box::new(DisallowedLevel {
                            level: request.level,
                        }));
                    }
                    Ok(TypedErrorsCookbookRequest::SetLevel(request.level))
                },
            ),
        )
    }
}

impl Domain for TypedErrorsCookbook {
    type Snapshot = TypedErrorsCookbookSnapshot;
    type Request = TypedErrorsCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut TypedErrorsCookbookSnapshot,
        command: Command<TypedErrorsCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(TypedErrorsCookbookRequest::SetLevel(level)) = command;
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
async fn wiring_refusal_uses_the_error_display() {
    let harness = Harness::<TypedErrorsCookbookService>::start(TypedErrorsCookbookArguments)
        .await
        .unwrap();
    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 13 })
        .await
        .unwrap();
    assert!(!ack.accepted);
    assert_eq!(ack.reason, "level 13 is not allowed");
}
