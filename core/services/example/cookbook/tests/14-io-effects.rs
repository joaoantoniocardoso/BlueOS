//! Device IO runs through Effects and reports back as IO results. The device is a Port in the Context: `context`
//! fills it with the real adapter, and a test replaces it through `Harness::start_with`.

use core::convert::Infallible;
use std::sync::Arc;

use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, LevelResponse};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

/// Reads the level sensor: the Port a test replaces.
type ReadSensor = Arc<dyn Fn() -> u8 + Send + Sync>;

struct IoCookbookService;

#[derive(Clone, Default, clap::Args)]
struct IoCookbookArguments;

struct IoCookbookContext {
    read_sensor: ReadSensor,
}

struct IoCookbook;

#[derive(Clone, Default)]
struct IoCookbookSnapshot {
    level: u8,
}

enum IoCookbookRequest {
    ReadLevel,
}

#[derive(Clone, Debug, PartialEq)]
enum IoCookbookIoRequest {
    ReadSensor,
}

#[derive(Clone, Debug, PartialEq)]
enum IoCookbookIoResult {
    Level(u8),
}

impl Service for IoCookbookService {
    type Domain = IoCookbook;
    type Context = IoCookbookContext;
    type Arguments = IoCookbookArguments;

    const NAME: &'static str = "cookbook_io";
    const VERSION: &'static str = "1.0.0";

    fn context(
        _service: &ServiceContext<IoCookbookArguments>,
    ) -> Result<IoCookbookContext, ServiceError> {
        Ok(IoCookbookContext {
            read_sensor: Arc::new(|| 42),
        })
    }

    fn build(
        _service: &ServiceContext<IoCookbookArguments>,
        _context: &IoCookbookContext,
    ) -> Result<ServiceBuilder<IoCookbook, IoCookbookContext>, ServiceError> {
        Ok(ServiceBuilder::new(IoCookbookSnapshot::default())
            .io(|io_context: &IoCookbookContext, _snapshot, request| {
                let level = match request {
                    IoCookbookIoRequest::ReadSensor => (io_context.read_sensor)(),
                };
                async move { Ok(Some(IoCookbookIoResult::Level(level))) }
            })
            .command("ReadLevel", |_: LevelRequest| {
                Ok(IoCookbookRequest::ReadLevel)
            })
            .state("pump", |snapshot: &IoCookbookSnapshot| LevelResponse {
                level: snapshot.level,
                max_level: 100,
            }))
    }
}

impl Domain for IoCookbook {
    type Snapshot = IoCookbookSnapshot;
    type Request = IoCookbookRequest;
    type Event = Infallible;
    type IoResult = IoCookbookIoResult;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = IoCookbookIoRequest;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut IoCookbookSnapshot,
        command: Command<IoCookbookRequest, IoCookbookIoResult, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(IoCookbookRequest::ReadLevel) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Io(IoCookbookIoRequest::ReadSensor)],
            },
            Command::IoResult(IoCookbookIoResult::Level(level)) => {
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
        request: IoCookbookIoRequest,
        error: IoError,
    ) -> Command<IoCookbookRequest, IoCookbookIoResult, Infallible, Infallible> {
        let _reason = error.message();
        match request {
            IoCookbookIoRequest::ReadSensor => Command::IoResult(IoCookbookIoResult::Level(0)),
        }
    }
}

#[tokio::test(start_paused = true)]
async fn io_effect_updates_the_snapshot() {
    let harness = Harness::<IoCookbookService>::start(IoCookbookArguments)
        .await
        .unwrap();
    let ack = harness
        .send("ReadLevel", &LevelRequest::default())
        .await
        .unwrap();
    assert!(ack.accepted);
    assert_eq!(
        harness.state::<LevelResponse>("pump").await.unwrap().level,
        42
    );
}

#[tokio::test(start_paused = true)]
async fn a_test_replaces_the_sensor_through_the_context() {
    let harness = Harness::<IoCookbookService>::start_with(IoCookbookArguments, |context| {
        context.read_sensor = Arc::new(|| 7);
    })
    .await
    .unwrap();
    let ack = harness
        .send("ReadLevel", &LevelRequest::default())
        .await
        .unwrap();
    assert!(ack.accepted);
    assert_eq!(
        harness.state::<LevelResponse>("pump").await.unwrap().level,
        7
    );
}
