//! The Snapshot is the single source of truth between Commands.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::SetLevelRequest;
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct HoldStateCookbookService;

#[derive(Clone, Default, clap::Args)]
struct HoldStateCookbookArguments;

struct HoldStateCookbook;

#[derive(Clone, Default)]
struct HoldStateCookbookSnapshot {
    level: u8,
    changes: u8,
}

enum HoldStateCookbookRequest {
    SetLevel(u8),
}

impl Service for HoldStateCookbookService {
    type Domain = HoldStateCookbook;
    type Context = ();
    type Arguments = HoldStateCookbookArguments;

    const NAME: &'static str = "cookbook_hold_state";
    const VERSION: &'static str = "1.0.0";

    fn build(
        _context: &ServiceContext<HoldStateCookbookArguments>,
    ) -> Result<ServiceBuilder<HoldStateCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(HoldStateCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelRequest| {
                Ok(HoldStateCookbookRequest::SetLevel(request.level))
            })
            .state("changes", |snapshot: &HoldStateCookbookSnapshot| {
                SetLevelRequest {
                    level: snapshot.changes,
                }
            }))
    }
}

impl Domain for HoldStateCookbook {
    type Snapshot = HoldStateCookbookSnapshot;
    type Request = HoldStateCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut HoldStateCookbookSnapshot,
        command: Command<HoldStateCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(HoldStateCookbookRequest::SetLevel(level)) = command;
        snapshot.level = level;
        snapshot.changes += 1;
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
async fn snapshot_accumulates_across_commands() {
    let harness = Harness::<HoldStateCookbookService>::start(HoldStateCookbookArguments)
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelRequest { level: 1 })
        .await;
    harness
        .send("SetLevel", &SetLevelRequest { level: 2 })
        .await;
    let changes = harness.state::<SetLevelRequest>("changes").await;
    assert_eq!(changes.level, 2);
}
