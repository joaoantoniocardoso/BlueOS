//! Structured `tracing` fields reach the standard `log` key.

use core::{convert::Infallible, time::Duration};
use std::sync::Arc;

use tokio_util::{sync::CancellationToken, task::TaskTracker};

use blueos_api::{Message, log_key};
use blueos_comms::channel::ChannelBackend;
use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::{blueos_example_msgs::EmptyRequest, foxglove_msgs::Log};
use blueos_logging::{attach, init};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct LoggingCookbookService;

#[derive(Clone, Default, clap::Args)]
struct LoggingCookbookArguments;

struct LoggingCookbook;

#[derive(Clone, Default)]
struct LoggingCookbookSnapshot;

enum LoggingCookbookRequest {
    Ping,
}

#[derive(Clone, Debug, PartialEq)]
enum LoggingCookbookIoRequest {
    Ping,
}

#[derive(Clone, Debug, PartialEq)]
enum LoggingCookbookIoResult {
    Done,
}

impl Service for LoggingCookbookService {
    type Domain = LoggingCookbook;
    type Context = ();
    type Arguments = LoggingCookbookArguments;

    const NAME: &'static str = "cookbook_logging";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<LoggingCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<LoggingCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<LoggingCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(LoggingCookbookSnapshot)
            .io(|_io_context, _snapshot, request| async move {
                match request {
                    LoggingCookbookIoRequest::Ping => {
                        tracing::info!(sensor = "demo", "device ping");
                        Ok(Some(LoggingCookbookIoResult::Done))
                    }
                }
            })
            .command("Ping", |_: EmptyRequest| Ok(LoggingCookbookRequest::Ping)))
    }
}

impl Domain for LoggingCookbook {
    type Snapshot = LoggingCookbookSnapshot;
    type Request = LoggingCookbookRequest;
    type Event = Infallible;
    type IoResult = LoggingCookbookIoResult;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = LoggingCookbookIoRequest;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut LoggingCookbookSnapshot,
        command: Command<LoggingCookbookRequest, LoggingCookbookIoResult, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(LoggingCookbookRequest::Ping) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Io(LoggingCookbookIoRequest::Ping)],
            },
            Command::IoResult(LoggingCookbookIoResult::Done) => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
            Command::Tick(never) => match never {},
            Command::ObservedFact(never) => match never {},
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {
            LoggingCookbookIoRequest::Ping => Command::IoResult(LoggingCookbookIoResult::Done),
        }
    }
}

#[tokio::test(start_paused = true)]
async fn structured_fields_appear_on_the_log_key() {
    init(0);
    let backend: Arc<dyn blueos_comms::CommsBackend> = Arc::new(ChannelBackend::default());
    let key = log_key(LoggingCookbookService::NAME);
    let mut subscriber = backend.subscribe(&key).await.expect("subscribe");
    let publisher = attach(Arc::clone(&backend), key).await;
    let shutdown = CancellationToken::new();
    let tasks = TaskTracker::new();
    tasks.spawn(publisher.run(shutdown.clone()));

    let harness = Harness::<LoggingCookbookService>::start_on(backend, LoggingCookbookArguments)
        .await
        .unwrap();
    harness.send("Ping", &EmptyRequest::default()).await;

    let sample = tokio::time::timeout(Duration::from_secs(1), subscriber.recv())
        .await
        .expect("log sample")
        .expect("sample");
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode");
    assert!(decoded.message.contains("device ping"));
    assert!(decoded.message.contains("sensor=demo"));

    shutdown.cancel();
    tasks.close();
    tasks.wait().await;
}
