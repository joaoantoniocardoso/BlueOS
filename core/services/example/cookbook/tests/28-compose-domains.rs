//! Compose reusable Blocks inside one Domain with [`Outcome::map`]. You do not compose two Domains in one
//! Service; you nest Blocks and map their [`Outcome`] into the parent Domain's events and effects.

use core::{
    convert::Infallible,
    error::Error,
    fmt::{self, Display, Formatter},
    time::Duration,
};

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, LevelResponse};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

const NOW: Now = Now {
    wall: Duration::from_secs(1_700_000_000),
    monotonic: Duration::from_secs(42),
};

type LampOutcome = Outcome<LampEvent, Infallible, Infallible, Infallible>;

#[derive(Clone, Default)]
struct RoomSnapshot {
    lamp: Lamp,
}

enum RoomRequest {
    TurnOn,
}

#[derive(Debug, PartialEq)]
enum RoomEvent {
    Lamp(LampEvent),
}

#[derive(Debug, PartialEq)]
enum LampEvent {
    TurnedOn,
}

#[derive(Clone, Default)]
enum Lamp {
    #[default]
    Off,
    On,
}

#[derive(Debug, PartialEq)]
enum LampRejection {
    AlreadyOn,
}

struct Room;

impl Domain for Room {
    type Snapshot = RoomSnapshot;
    type Request = RoomRequest;
    type Event = RoomEvent;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut RoomSnapshot,
        command: Command<RoomRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(RoomRequest::TurnOn) => snapshot.lamp.turn_on().map(
                RoomEvent::Lamp,
                |tick: Infallible| match tick {},
                |io: Infallible| match io {},
                |key: Infallible| match key {},
            ),
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

impl Lamp {
    fn turn_on(&mut self) -> LampOutcome {
        if matches!(self, Self::On) {
            return Outcome::reject(LampRejection::AlreadyOn);
        }
        *self = Self::On;
        Outcome::Applied {
            events: vec![LampEvent::TurnedOn],
            effects: Vec::new(),
        }
    }
}

impl Display for LampRejection {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyOn => formatter.write_str("the lamp is already on"),
        }
    }
}

impl Error for LampRejection {}

struct ComposeCookbookService;

#[derive(Clone, Default, clap::Args)]
struct ComposeCookbookArguments;

impl Service for ComposeCookbookService {
    type Domain = Room;
    type Context = ();
    type Arguments = ComposeCookbookArguments;

    const NAME: &'static str = "cookbook_compose";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<ComposeCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<ComposeCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Room>, ServiceError> {
        Ok(ServiceBuilder::new(RoomSnapshot::default())
            .command("TurnOn", |_: LevelRequest| Ok(RoomRequest::TurnOn))
            .state("lamp", |snapshot: &RoomSnapshot| LevelResponse {
                level: u8::from(matches!(snapshot.lamp, Lamp::On)),
                max_level: 1,
            }))
    }
}

#[test]
fn block_outcome_maps_into_the_composing_domain() {
    let mut snapshot = RoomSnapshot::default();
    let decision = Room::handle(&mut snapshot, Command::Request(RoomRequest::TurnOn), NOW);
    let Outcome::Applied { events, effects } = decision else {
        panic!("turning the lamp on must apply, got {decision:?}");
    };
    assert_eq!(events, vec![RoomEvent::Lamp(LampEvent::TurnedOn)]);
    assert!(effects.is_empty());
    assert!(matches!(snapshot.lamp, Lamp::On));
}

#[test]
fn block_rejection_maps_into_the_composing_domain() {
    let mut snapshot = RoomSnapshot { lamp: Lamp::On };
    let decision = Room::handle(&mut snapshot, Command::Request(RoomRequest::TurnOn), NOW);
    let Outcome::Rejected { reason } = decision else {
        panic!("turning an on lamp on must reject, got {decision:?}");
    };
    assert_eq!(reason.downcast_ref(), Some(&LampRejection::AlreadyOn));
}

#[tokio::test(start_paused = true)]
async fn composed_block_state_reaches_clients_through_the_service() {
    let harness = Harness::<ComposeCookbookService>::start(ComposeCookbookArguments)
        .await
        .expect("start");
    let ack = harness.send("TurnOn", &LevelRequest::default()).await;
    assert!(ack.accepted);
    let lamp = harness.state::<LevelResponse>("lamp").await;
    assert_eq!(lamp.level, 1);
}
