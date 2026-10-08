//! Question 14: how do I do device IO, and replace the device in a test?
//!
//! The answer is the `Effect::Io` returned by the `ReadLevel` arm of `Domain::handle` and the `.io(...)` handler in
//! `build` (the IO), the `IoCookbookContext` Port (the device), and `a_test_replaces_the_sensor_through_the_context`
//! (the replacement). Shared boilerplate is explained in `01-command.rs`.
//!
//! The Domain never touches the device (D-03, D-25): it asks for IO as an Effect, and the Kernel runs the handler
//! and feeds the answer back as a `Command::IoResult`.

use core::convert::Infallible;
use std::sync::Arc;

use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, LevelResponse};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

// A Port is a field of the Context holding the capability to reach a device, so a test can swap it (D-25).
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

// The IoRequest names what IO the Domain wants and the IoResult what came back; both are the Domain's own types,
// so it stays free of the device and of async (D-03).
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
        // The real adapter goes here; a test overrides it after this runs and before `build`.
        Ok(IoCookbookContext {
            read_sensor: Arc::new(|| 42),
        })
    }

    fn build(
        _service: &ServiceContext<IoCookbookArguments>,
        _context: &IoCookbookContext,
    ) -> Result<ServiceBuilder<IoCookbook, IoCookbookContext>, ServiceError> {
        Ok(ServiceBuilder::new(IoCookbookSnapshot::default())
            // The one place IO happens: it reads the Port from the Context, never from the Domain. Returning
            // `Ok(Some(..))` reports the result back to `handle`; an `Err` goes to `io_failed` instead.
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
            // The Request only asks for the read; the Snapshot changes when the result arrives below.
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
        // A failed IO becomes an ordinary Command, so the Domain decides what failure means (here, level 0).
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
    // `start_with` edits the Context between `context` and `build`, so the test runs the shipped wiring with
    // only the device swapped.
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
    // 7 instead of the real adapter's 42 proves the Domain's IO went through the replaced Port.
    assert_eq!(
        harness.state::<LevelResponse>("pump").await.unwrap().level,
        7
    );
}
