//! Inbox loop panics are logged on the service `log` key (D-29).

use core::{convert::Infallible, time::Duration};
use std::sync::Arc;

use clap::Args;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use blueos_api::{Message, log_key};
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_domain::{Command, Decision, Domain, Now, Outcome};
use blueos_idl::{msg::blueos_example_msgs::SetLevelGoal, msg::foxglove_msgs::Log};
use blueos_logging::{attach, init};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

const RECV_TIMEOUT: Duration = Duration::from_secs(10);
const LEVEL_THAT_PANICS_IN_HANDLE: u8 = 99;

struct TankService;

#[derive(Args, Clone)]
struct TankArguments {
    #[arg(long, default_value_t = 100)]
    capacity: u8,
}

struct Tank;

#[derive(Clone)]
struct TankSnapshot {
    level: u8,
}

enum TankRequest {
    SetLevel(u8),
}

enum TankEvent {
    LevelChanged,
}

impl Service for TankService {
    type Domain = Tank;
    type Context = ();
    type Arguments = TankArguments;

    const NAME: &'static str = "logging-recovery-inbox";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TankArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<TankArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        Ok(ServiceBuilder::new(TankSnapshot { level: 0 })
            .command("SetLevel", |request: SetLevelGoal| {
                Ok(TankRequest::SetLevel(request.level))
            }))
    }
}

impl Domain for Tank {
    type Snapshot = TankSnapshot;
    type Request = TankRequest;
    type Event = TankEvent;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut TankSnapshot,
        command: Command<TankRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let TankRequest::SetLevel(level) = match command {
            Command::Request(request) => request,
            Command::IoResult(_) | Command::Tick(_) | Command::ObservedFact(_) => {
                unreachable!();
            }
        };
        if level == LEVEL_THAT_PANICS_IN_HANDLE {
            panic!("inbox loop panic marker");
        }
        snapshot.level = level;
        Outcome::Applied {
            events: vec![TankEvent::LevelChanged],
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

#[tokio::test(start_paused = true)]
async fn inbox_loop_panic_message_reaches_log_key() {
    init(0);
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let key = log_key(TankService::NAME);
    let mut subscriber = backend.subscribe(&key).await.expect("subscribe");
    let publisher = attach(Arc::clone(&backend), key).await;
    let shutdown = CancellationToken::new();
    let tasks = TaskTracker::new();
    tasks.spawn(publisher.run(shutdown.clone()));

    let harness =
        Harness::<TankService>::start_on(Arc::clone(&backend), TankArguments { capacity: 100 })
            .await
            .expect("harness starts");

    harness
        .send(
            "SetLevel",
            &SetLevelGoal {
                level: LEVEL_THAT_PANICS_IN_HANDLE,
            },
        )
        .await;

    let sample = tokio::time::timeout(RECV_TIMEOUT, subscriber.recv())
        .await
        .expect("log sample arrives")
        .expect("sample");
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert!(decoded.message.contains("inbox loop panic marker"));
}
