//! Question 8: how do I hold state between Commands?
//!
//! The answer is `Domain::handle`: it mutates the Snapshot it is given, and the Snapshot is the only thing that
//! survives from one Command to the next. Shared boilerplate: `01-command.rs`.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::SetLevelGoal;
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

    fn context(_service: &ServiceContext<HoldStateCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<HoldStateCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<HoldStateCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(HoldStateCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelGoal| {
                Ok(HoldStateCookbookRequest::SetLevel(request.level))
            })
            .state("changes", |snapshot: &HoldStateCookbookSnapshot| {
                SetLevelGoal {
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
        // State lives in the Snapshot, never in the marker type or a static (D-03, D-25): that keeps `handle` a pure
        // function of its inputs, and lets the Kernel clone the Snapshot to roll back a failed Command (D-04).
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
        .send("SetLevel", &SetLevelGoal { level: 1 })
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelGoal { level: 2 })
        .await
        .unwrap();
    // Two Commands, one counter: only a Snapshot that persisted between them can read 2.
    let changes = harness.state::<SetLevelGoal>("changes").await.unwrap();
    assert_eq!(changes.level, 2);
}
