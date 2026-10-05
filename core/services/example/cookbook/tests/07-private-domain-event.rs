//! Domain events that never select a public Event endpoint stay on the bus.

use core::{convert::Infallible, time::Duration};

use tokio::time::timeout;

use blueos_api::event_key;
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, SetLevelGoal};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct PrivateEventCookbookService;

#[derive(Clone, Default, clap::Args)]
struct PrivateEventCookbookArguments;

struct PrivateEventCookbook;

#[derive(Clone, Default)]
struct PrivateEventCookbookSnapshot {
    level: u8,
}

enum PrivateEventCookbookRequest {
    SetLevel(u8),
    Bump,
}

enum PrivateEventCookbookEvent {
    LevelChanged(u8),
    InternalNote,
}

impl Service for PrivateEventCookbookService {
    type Domain = PrivateEventCookbook;
    type Context = ();
    type Arguments = PrivateEventCookbookArguments;

    const NAME: &'static str = "cookbook_private_event";
    const VERSION: &'static str = "1.0.0";

    fn context(
        _service: &ServiceContext<PrivateEventCookbookArguments>,
    ) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<PrivateEventCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<PrivateEventCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(PrivateEventCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelGoal| {
                Ok(PrivateEventCookbookRequest::SetLevel(request.level))
            })
            .command("Bump", |_: LevelRequest| {
                Ok(PrivateEventCookbookRequest::Bump)
            })
            .event(
                "LevelChanged",
                |event: &PrivateEventCookbookEvent| match event {
                    PrivateEventCookbookEvent::LevelChanged(level) => {
                        Some(SetLevelGoal { level: *level })
                    }
                    PrivateEventCookbookEvent::InternalNote => None,
                },
            ))
    }
}

impl Domain for PrivateEventCookbook {
    type Snapshot = PrivateEventCookbookSnapshot;
    type Request = PrivateEventCookbookRequest;
    type Event = PrivateEventCookbookEvent;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut PrivateEventCookbookSnapshot,
        command: Command<PrivateEventCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(PrivateEventCookbookRequest::SetLevel(level)) => {
                snapshot.level = level;
                Outcome::Applied {
                    events: vec![
                        PrivateEventCookbookEvent::LevelChanged(level),
                        PrivateEventCookbookEvent::InternalNote,
                    ],
                    effects: Vec::new(),
                }
            }
            Command::Request(PrivateEventCookbookRequest::Bump) => Outcome::Applied {
                events: vec![PrivateEventCookbookEvent::InternalNote],
                effects: Vec::new(),
            },
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
async fn private_domain_events_do_not_publish() {
    let harness = Harness::<PrivateEventCookbookService>::start(PrivateEventCookbookArguments)
        .await
        .unwrap();
    let mut events = harness
        .backend()
        .subscribe(&event_key(
            PrivateEventCookbookService::NAME,
            "LevelChanged",
        ))
        .await
        .unwrap();
    harness.send("Bump", &LevelRequest::default()).await;
    let maybe = timeout(Duration::from_secs(1), events.recv()).await;
    assert!(maybe.is_err(), "InternalNote must not publish LevelChanged");
}
