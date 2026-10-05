//! Recovery integration-test fixtures.

#![expect(
    unreachable_pub,
    reason = "fixture items are re-exported to integration test roots"
)]
#![expect(
    dead_code,
    reason = "fixture items are shared across sibling integration test binaries"
)]

use core::{convert::Infallible, time::Duration};

use clap::Args;
use tokio::time::timeout;

use blueos_domain::{Command, Decision, Domain, Now, Outcome};
use blueos_idl::{
    Message,
    msg::{
        blueos_example_msgs::{PumpState, SetLevelGoal},
        blueos_msgs::ServiceStatus,
    },
};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError};

pub const RECV_TIMEOUT: Duration = Duration::from_secs(10);
pub const LEVEL_THAT_PANICS_IN_HANDLE: u8 = 99;

pub struct TankService;

#[derive(Args, Clone)]
pub struct TankArguments {
    #[arg(long, default_value_t = 100)]
    pub capacity: u8,
}

pub struct Tank;

#[derive(Clone)]
pub struct TankSnapshot {
    level: u8,
    capacity: u8,
    level_set_at: Duration,
}

pub enum TankRequest {
    SetLevel(u8),
}

pub enum TankEvent {
    LevelChanged,
}

pub struct TasksService;

#[derive(Args, Clone)]
pub struct TasksArguments {}

pub struct TasksDomain;

#[derive(Clone, Default)]
pub struct TasksSnapshot;

impl Service for TankService {
    type Domain = Tank;
    type Context = ();
    type Arguments = TankArguments;

    const NAME: &'static str = "recovery-tank";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TankArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<TankArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        let capacity = service.arguments().capacity;
        Ok(ServiceBuilder::new(TankSnapshot {
            level: 0,
            capacity,
            level_set_at: Duration::ZERO,
        })
        .command("SetLevel", |request: SetLevelGoal| {
            Ok(TankRequest::SetLevel(request.level))
        })
        .state("tank", |snapshot: &TankSnapshot| PumpState {
            level: snapshot.level,
            max_level: snapshot.capacity,
            ..PumpState::default()
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
        now: Now,
    ) -> Decision<Self> {
        let TankRequest::SetLevel(level) = match command {
            Command::Request(request) => request,
            Command::IoResult(_) | Command::Tick(_) | Command::ObservedFact(_) => {
                unreachable!("the recovery tank has no IO, timers or observed facts");
            }
        };
        if level == LEVEL_THAT_PANICS_IN_HANDLE {
            panic!("handle panicked");
        }
        snapshot.level = level;
        snapshot.level_set_at = now.monotonic;
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

impl Service for TasksService {
    type Domain = TasksDomain;
    type Context = ();
    type Arguments = TasksArguments;

    const NAME: &'static str = "recovery-tasks";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TasksArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<TasksArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<TasksDomain>, ServiceError> {
        Ok(ServiceBuilder::new(TasksSnapshot))
    }
}

impl Domain for TasksDomain {
    type Snapshot = TasksSnapshot;
    type Request = Infallible;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut TasksSnapshot,
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

pub async fn next_status(subscriber: &mut blueos_comms::Subscriber) -> ServiceStatus {
    let sample = timeout(RECV_TIMEOUT, subscriber.recv())
        .await
        .expect("status update arrives before timeout")
        .expect("status stream stays open");
    ServiceStatus::decode(&sample.payload().to_bytes()).expect("status payload decodes")
}
