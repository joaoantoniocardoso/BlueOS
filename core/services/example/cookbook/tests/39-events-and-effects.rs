//! Question 39: how do I announce what happened and ask for IO in the same Command?
//!
//! The answer is the `Outcome::Applied` of the `Measure` arm of `Domain::handle`, which holds both an Event and an
//! Effect, and `an_outcome_publishes_its_event_and_runs_its_io`, which observes both. Shared boilerplate is
//! explained in `01-command.rs`, Events in `06-event.rs` and IO in `14-io-effects.rs`.
//!
//! The Kernel applies the Outcome in one go: the Snapshot change is committed, the Event is published, and the
//! Effect is run. The IO answer comes back later as its own Command (`IoResult`), so the Event never carries it.

use core::{convert::Infallible, time::Duration};
use std::sync::Arc;

use tokio::time::timeout;

use blueos_api::{Message, event_key};
use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, LevelResponse};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

type ReadSensor = Arc<dyn Fn() -> u8 + Send + Sync>;

struct BothCookbookService;

#[derive(Clone, Default, clap::Args)]
struct BothCookbookArguments;

struct BothCookbookContext {
    read_sensor: ReadSensor,
}

struct BothCookbook;

#[derive(Clone, Default)]
struct BothCookbookSnapshot {
    level: u8,
}

enum BothCookbookRequest {
    Measure,
}

enum BothCookbookEvent {
    MeasurementStarted,
}

#[derive(Clone, Debug, PartialEq)]
enum BothCookbookIoRequest {
    ReadSensor,
}

#[derive(Clone, Debug, PartialEq)]
enum BothCookbookIoResult {
    Level(u8),
}

impl Service for BothCookbookService {
    type Domain = BothCookbook;
    type Context = BothCookbookContext;
    type Arguments = BothCookbookArguments;

    const NAME: &'static str = "cookbook_both";
    const VERSION: &'static str = "1.0.0";

    fn context(
        _service: &ServiceContext<BothCookbookArguments>,
    ) -> Result<BothCookbookContext, ServiceError> {
        Ok(BothCookbookContext {
            read_sensor: Arc::new(|| 42),
        })
    }

    fn build(
        _service: &ServiceContext<BothCookbookArguments>,
        _context: &BothCookbookContext,
    ) -> Result<ServiceBuilder<BothCookbook, BothCookbookContext>, ServiceError> {
        Ok(ServiceBuilder::new(BothCookbookSnapshot::default())
            .io(|io_context: &BothCookbookContext, _snapshot, request| {
                let level = match request {
                    BothCookbookIoRequest::ReadSensor => (io_context.read_sensor)(),
                };
                async move { Ok(Some(BothCookbookIoResult::Level(level))) }
            })
            .command("Measure", |_: LevelRequest| {
                Ok(BothCookbookRequest::Measure)
            })
            .event(
                "MeasurementStarted",
                |event: &BothCookbookEvent| match event {
                    BothCookbookEvent::MeasurementStarted => Some(LevelResponse {
                        level: 0,
                        max_level: 100,
                    }),
                },
            )
            .state("pump", |snapshot: &BothCookbookSnapshot| LevelResponse {
                level: snapshot.level,
                max_level: 100,
            }))
    }
}

impl Domain for BothCookbook {
    type Snapshot = BothCookbookSnapshot;
    type Request = BothCookbookRequest;
    type Event = BothCookbookEvent;
    type IoResult = BothCookbookIoResult;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = BothCookbookIoRequest;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut BothCookbookSnapshot,
        command: Command<BothCookbookRequest, BothCookbookIoResult, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            // One Outcome announces what happened (the Event) and asks for what is needed next (the Effect).
            Command::Request(BothCookbookRequest::Measure) => Outcome::Applied {
                events: vec![BothCookbookEvent::MeasurementStarted],
                effects: vec![Effect::Io(BothCookbookIoRequest::ReadSensor)],
            },
            Command::IoResult(BothCookbookIoResult::Level(level)) => {
                snapshot.level = level;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Tick(never) => match never {},
            Command::ObservedFact(never) => match never {},
        }
    }

    fn io_failed(
        request: BothCookbookIoRequest,
        _error: IoError,
    ) -> Command<BothCookbookRequest, BothCookbookIoResult, Infallible, Infallible> {
        match request {
            BothCookbookIoRequest::ReadSensor => Command::IoResult(BothCookbookIoResult::Level(0)),
        }
    }
}

#[tokio::test(start_paused = true)]
async fn an_outcome_publishes_its_event_and_runs_its_io() {
    // The device is replaced through the Context, as in `14-io-effects.rs`.
    let harness = Harness::<BothCookbookService>::start_with(BothCookbookArguments, |context| {
        context.read_sensor = Arc::new(|| 7);
    })
    .await
    .unwrap();
    // Subscribe before sending: an Event is not retained.
    let mut events = harness
        .backend()
        .subscribe(&event_key(BothCookbookService::NAME, "MeasurementStarted"))
        .await
        .unwrap();
    let ack = harness
        .send("Measure", &LevelRequest::default())
        .await
        .unwrap();
    assert!(ack.accepted);

    let sample = timeout(Duration::from_secs(10), events.recv())
        .await
        .unwrap()
        .unwrap();
    // The subscriber sees the announcement only; the IO answer is not in it.
    let decoded =
        LevelResponse::decode(sample.payload().to_bytes().as_ref()).expect("decode event");
    assert_eq!(decoded.level, 0);
    // The Effect ran against the replaced device: 7, not the real adapter's 42.
    assert_eq!(
        harness.state::<LevelResponse>("pump").await.unwrap().level,
        7
    );
}
