//! L5: a Service on `zenohd` serves its States after startup when configured.

use core::{convert::Infallible, time::Duration};
use std::sync::Arc;

use clap::Args;

use blueos_api::{Message, state_key};
use blueos_comms::CommsBackend;
use blueos_comms_zenoh::ZenohBackend;
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::PumpState;
use blueos_service::{
    Kernel, Service, ServiceBuilder, ServiceContext, ServiceError, testing::PausedClock,
};

struct ZenohStartupService;

#[derive(Args, Clone)]
struct ZenohStartupArguments {}

struct ZenohStartup;

#[derive(Clone, Default)]
struct ZenohStartupSnapshot {
    ready: bool,
}

impl Service for ZenohStartupService {
    type Domain = ZenohStartup;
    type Context = ();
    type Arguments = ZenohStartupArguments;

    const NAME: &'static str = "zenoh_startup";
    const VERSION: &'static str = "1.0.0";

    fn build(
        _context: &ServiceContext<ZenohStartupArguments>,
    ) -> Result<ServiceBuilder<ZenohStartup>, ServiceError> {
        Ok(
            ServiceBuilder::new(ZenohStartupSnapshot { ready: true }).state(
                "ready",
                |snapshot: &ZenohStartupSnapshot| PumpState {
                    level: u8::from(snapshot.ready),
                    max_level: 1,
                    ..PumpState::default()
                },
            ),
        )
    }
}

impl Domain for ZenohStartup {
    type Snapshot = ZenohStartupSnapshot;
    type Request = Infallible;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut Self::Snapshot,
        _command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        _now: Now,
    ) -> Decision<Self> {
        Outcome::Applied {
            events: Vec::new(),
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn zenoh_startup_serves_state_after_kernel_start() {
    let Some(endpoint) = std::env::var("BLUEOS_ZENOH_ENDPOINT").ok() else {
        eprintln!(
            "skip: set BLUEOS_ZENOH_ENDPOINT (for example tcp/127.0.0.1:7447) to run zenoh startup conformance"
        );
        return;
    };
    let backend: Arc<dyn CommsBackend> = Arc::new(
        ZenohBackend::connect(&endpoint)
            .await
            .unwrap_or_else(|error| panic!("could not connect to zenohd at {endpoint}: {error}")),
    );
    let client = Arc::clone(&backend);
    let kernel = Kernel::start(
        ZenohStartupService::NAME,
        ZenohStartupService::build(&ServiceContext::new(ZenohStartupArguments {})).unwrap(),
        backend,
        Arc::new(PausedClock::start()),
    )
    .await
    .expect("start");
    let _run = tokio::spawn(kernel.run());
    let key = state_key(ZenohStartupService::NAME, "ready");
    let replies = client
        .get(&key, None, Duration::from_secs(5))
        .await
        .expect("get");
    assert_eq!(replies.len(), 1);
    let sample = replies[0].as_ref().expect("reply");
    let state = PumpState::decode(&sample.payload().to_bytes()).expect("PumpState");
    assert_eq!(state.level, 1);
}
