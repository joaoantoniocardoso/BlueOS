//! A log line emitted while a Task shuts down is published before `Kernel::run` returns.

use core::{convert::Infallible, time::Duration};
use std::sync::Arc;

use clap::Args;
use tokio::time::{advance, timeout};

use blueos_api::log_key;
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_domain::{Command, Decision, Domain, Now, Outcome};
use blueos_idl::{Message, msg::foxglove_msgs::Log};
use blueos_logging::{attach, init};
use blueos_service::{
    Kernel, LogPublisherRuntime, RestartPolicy, RunOutcome, Service, ServiceBuilder,
    ServiceContext, ServiceError, testing::PausedClock,
};

const RECV_TIMEOUT: Duration = Duration::from_secs(10);

struct ShutdownLogService;

#[derive(Args, Clone)]
struct ShutdownLogArguments {}

struct ShutdownLogDomain;

#[derive(Clone, Default)]
struct ShutdownLogSnapshot;

impl Service for ShutdownLogService {
    type Domain = ShutdownLogDomain;
    type Context = ();
    type Arguments = ShutdownLogArguments;

    const NAME: &'static str = "logging-shutdown-task";
    const VERSION: &'static str = "1.0.0";

    fn build(
        _context: &ServiceContext<ShutdownLogArguments>,
    ) -> Result<ServiceBuilder<ShutdownLogDomain>, ServiceError> {
        Ok(ServiceBuilder::new(ShutdownLogSnapshot).task(
            "logger",
            RestartPolicy::Never,
            |task_context| async move {
                task_context.shutdown.cancelled().await;
                tracing::warn!(phase = "task_shutdown", "Task left its loop");
                Ok(())
            },
        ))
    }
}

impl Domain for ShutdownLogDomain {
    type Snapshot = ShutdownLogSnapshot;
    type Request = Infallible;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut ShutdownLogSnapshot,
        _command: Command<Infallible, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: blueos_service::IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

#[tokio::test(start_paused = true)]
async fn task_shutdown_log_is_published_before_run_returns() {
    init(0);
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let key = log_key(ShutdownLogService::NAME);
    let mut subscriber = backend.subscribe(&key).await.expect("subscribe");
    let log_runtime = LogPublisherRuntime::start(attach(Arc::clone(&backend), key).await);

    let mut builder = ShutdownLogService::build(&ServiceContext::new(
        ShutdownLogArguments {},
        Arc::clone(&backend),
    ))
    .expect("build");
    let shutdown = builder.shutdown_handle();
    let mut kernel = Kernel::start(
        ShutdownLogService::NAME,
        builder,
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    )
    .await
    .expect("kernel starts");
    kernel.attach_log_publisher(log_runtime);
    let run = tokio::spawn(async move { kernel.run().await });

    shutdown.trigger();
    advance(Duration::from_secs(5)).await;

    let sample = timeout(RECV_TIMEOUT, subscriber.recv())
        .await
        .expect("log sample arrives before run returns")
        .expect("sample");
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert!(decoded.message.contains("phase=task_shutdown"));

    assert_eq!(run.await.expect("join"), RunOutcome::Stopped);
}
