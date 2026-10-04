//! A start failure after the Session opens is published on the `log` key before the process ends.

use core::{convert::Infallible, time::Duration};
use std::{ffi::OsString, sync::Arc};

use clap::Args;

use blueos_api::log_key;
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_domain::{Command, Decision, Domain, Now, Outcome};
use blueos_idl::{Message, msg::foxglove_msgs::Log};
use blueos_service::{
    Service, ServiceBuilder, ServiceContext, ServiceError,
    entry::{parse_service_cli, run_with_backend},
    testing::PausedClock,
};

struct RefusingService;

#[derive(Args, Clone)]
struct RefusingArguments {}

struct RefusingDomain;

#[derive(Clone, Default)]
struct RefusingSnapshot;

impl Service for RefusingService {
    type Domain = RefusingDomain;
    type Context = ();
    type Arguments = RefusingArguments;

    const NAME: &'static str = "logging-start-failure";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<RefusingArguments>) -> Result<(), ServiceError> {
        Err(ServiceError::Build("refused to start".into()))
    }

    fn build(
        _service: &ServiceContext<RefusingArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<RefusingDomain>, ServiceError> {
        Ok(ServiceBuilder::new(RefusingSnapshot))
    }
}

impl Domain for RefusingDomain {
    type Snapshot = RefusingSnapshot;
    type Request = Infallible;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut RefusingSnapshot,
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
        _error: blueos_domain::IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

#[test]
fn start_failure_is_published_before_the_runtime_ends() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let parsed =
        parse_service_cli::<RefusingService>(["logging-start-failure"].map(OsString::from))
            .expect("parse");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("runtime");
    let mut subscriber = runtime.block_on(async {
        blueos_logging::init(0);
        backend
            .subscribe(&log_key(RefusingService::NAME))
            .await
            .expect("subscribe")
    });

    let outcome = runtime.block_on(run_with_backend::<RefusingService>(
        parsed,
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    ));
    assert!(outcome.is_err());
    drop(runtime);

    let reader = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("runtime");
    let sample = reader
        .block_on(async {
            tokio::time::timeout(Duration::from_millis(50), subscriber.recv()).await
        })
        .expect("the start failure reaches the log key")
        .expect("sample");
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert!(
        decoded
            .message
            .contains("The service could not start or run")
    );
}
