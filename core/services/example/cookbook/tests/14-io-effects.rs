//! Device IO runs through Effects and reports back as IO results.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct IoCookbookService;

#[derive(Clone, Default, clap::Args)]
struct IoCookbookArguments;

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
    type Context = ();
    type Arguments = IoCookbookArguments;

    const NAME: &'static str = "cookbook_io";
    const VERSION: &'static str = "1.0.0";

    fn build(
        _context: &ServiceContext<IoCookbookArguments>,
    ) -> Result<ServiceBuilder<IoCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(IoCookbookSnapshot::default())
            .io(|_io_context, _snapshot, request| async move {
                match request {
                    IoCookbookIoRequest::ReadSensor => Ok(Some(IoCookbookIoResult::Level(42))),
                }
            })
            .command("ReadLevel", |_: EmptyRequest| {
                Ok(IoCookbookRequest::ReadLevel)
            })
            .state("pump", |snapshot: &IoCookbookSnapshot| LevelQueryResponse {
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
    let ack = harness.send("ReadLevel", &EmptyRequest::default()).await;
    assert!(ack.accepted);
    assert_eq!(harness.state::<LevelQueryResponse>("pump").await.level, 42);
}
