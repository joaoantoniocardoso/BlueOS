//! Emit a public Event after a Command is acknowledged.

use core::{convert::Infallible, time::Duration};

use tokio::time::timeout;

use blueos_api::{Message, event_key};
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelResponse, SetLevelGoal};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct EventCookbookService;

#[derive(Clone, Default, clap::Args)]
struct EventCookbookArguments;

struct EventCookbook;

#[derive(Clone, Default)]
struct EventCookbookSnapshot {
    level: u8,
}

enum EventCookbookRequest {
    SetLevel(u8),
}

enum EventCookbookEvent {
    LevelChanged(u8),
}

impl Service for EventCookbookService {
    type Domain = EventCookbook;
    type Context = ();
    type Arguments = EventCookbookArguments;

    const NAME: &'static str = "cookbook_event";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<EventCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<EventCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<EventCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(EventCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelGoal| {
                Ok(EventCookbookRequest::SetLevel(request.level))
            })
            .event("LevelChanged", |event: &EventCookbookEvent| match event {
                EventCookbookEvent::LevelChanged(level) => Some(LevelResponse {
                    level: *level,
                    max_level: 100,
                }),
            }))
    }
}

impl Domain for EventCookbook {
    type Snapshot = EventCookbookSnapshot;
    type Request = EventCookbookRequest;
    type Event = EventCookbookEvent;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut EventCookbookSnapshot,
        command: Command<EventCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(EventCookbookRequest::SetLevel(level)) = command;
        snapshot.level = level;
        Outcome::Applied {
            events: vec![EventCookbookEvent::LevelChanged(level)],
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
async fn event_arrives_after_the_ack() {
    let harness = Harness::<EventCookbookService>::start(EventCookbookArguments)
        .await
        .unwrap();
    let mut events = harness
        .backend()
        .subscribe(&event_key(EventCookbookService::NAME, "LevelChanged"))
        .await
        .unwrap();
    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 15 })
        .await
        .unwrap();
    assert!(ack.accepted);
    let sample = timeout(Duration::from_secs(10), events.recv())
        .await
        .unwrap()
        .unwrap();
    let decoded =
        LevelResponse::decode(sample.payload().to_bytes().as_ref()).expect("decode event");
    assert_eq!(decoded.level, 15);
}
