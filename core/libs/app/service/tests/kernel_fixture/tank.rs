use core::{convert::Infallible, sync::atomic::AtomicBool, time::Duration};
use std::sync::{Arc, Mutex};

use tokio::sync::Semaphore;

use blueos_api::Message;
use blueos_comms::channel::ChannelBackend;
use blueos_domain::{Command, Decision, Domain, DomainQueries, Now, Outcome};
use blueos_idl::{
    Error as IdlError,
    cdr::{Reader, Writer},
    message::CdrStruct,
    msg::{
        blueos_example_msgs::{LevelResponse, PumpState, SetLevelGoal},
        builtin_interfaces::Time,
        std_msgs::Empty,
    },
};
use blueos_service::{Refusal, Service, ServiceBuilder, ServiceContext, ServiceError};

/// `handle` panics after it changed the Snapshot.
pub const LEVEL_THAT_PANICS_IN_HANDLE: u8 = 99;
/// The `tank` State's projection panics.
pub const LEVEL_THAT_PANICS_IN_PROJECTION: u8 = 98;
/// The `FragileLevel` State fails to encode.
pub const LEVEL_THAT_FAILS_TO_ENCODE: u8 = 13;

/// A Service whose one Command sets the level of a tank.
pub struct TankService;

#[derive(clap::Args)]
pub struct TankArguments {
    #[arg(long, default_value_t = 100)]
    pub capacity: u8,
}

pub struct Tank;

#[derive(Clone)]
pub struct TankSnapshot {
    pub level: u8,
    pub capacity: u8,
    /// The wall-clock time of the last applied `SetLevel`.
    pub level_set_at: Duration,
}

pub enum TankRequest {
    SetLevel(u8),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TankTimerKey {}

pub enum TankEvent {
    LevelChanged(u8),
    Emptied,
}

pub enum TankQuery {
    Level,
    /// Answered with a Response that the `Level` conversion does not publish.
    Other,
    Panic,
}

pub enum TankResponse {
    Level(u8),
    Other,
}

#[derive(Debug, thiserror::Error)]
#[error("level {level} is above the capacity {capacity}")]
pub struct AboveCapacity {
    level: u8,
    capacity: u8,
}

#[derive(Debug, thiserror::Error)]
#[error("a tank needs a capacity above 0")]
pub struct NoCapacity;

#[derive(Debug, thiserror::Error)]
#[error("{0} is not a percentage")]
pub struct NotAPercent(pub u8);

/// The tank, with IO queries that read a simulated sensor outside the Inbox.
pub struct ProbeService;

#[derive(clap::Args, Default)]
pub struct ProbeArguments {
    #[arg(skip)]
    pub sensor: Sensor,
}

/// A simulated level sensor: each permit lets one `Probe` read it.
#[derive(Clone)]
pub struct Sensor(pub Arc<Semaphore>);

/// The tank, publishing a State that fails to encode at one level.
pub struct FragileTankService;

/// A State Message whose encoding fails at [`LEVEL_THAT_FAILS_TO_ENCODE`].
#[derive(Debug)]
pub struct FragileLevel {
    pub level: u8,
}

/// The tank, with a Command name that is not a valid key.
pub struct MisnamedTankService;

/// A backbone that closes every endpoint as soon as it is declared.
pub struct ClosedBackend;

/// The channel backend, recording every publish and every reply in order, and failing publishes on demand.
#[derive(Default)]
pub struct RecordingBackend {
    pub bus: ChannelBackend,
    pub journal: Arc<Mutex<Vec<String>>>,
    pub publishes_fail: AtomicBool,
}

impl CdrStruct for FragileLevel {
    fn cdr_decode_fields(reader: &mut Reader) -> Result<Self, IdlError> {
        Ok(Self {
            level: reader.read_u8()?,
        })
    }

    fn cdr_encode_fields(&self, writer: &mut Writer) -> Result<(), IdlError> {
        if self.level == LEVEL_THAT_FAILS_TO_ENCODE {
            return Err(IdlError::InvalidLength);
        }
        writer.write_u8(self.level)
    }
}

impl Message for FragileLevel {
    const SCHEMA: &'static str = "uint8 level";
    const SCHEMA_NAME: &'static str = "blueos_test_msgs/msg/FragileLevel";
    const TYPE_HASH: &'static str = "";
}

impl TankSnapshot {
    pub(crate) fn empty(capacity: u8) -> Self {
        Self {
            level: 0,
            capacity,
            level_set_at: Duration::ZERO,
        }
    }
}

impl Domain for Tank {
    type Snapshot = TankSnapshot;
    type Request = TankRequest;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type Event = TankEvent;
    type IoRequest = Infallible;
    type TimerKey = TankTimerKey;

    fn handle(
        snapshot: &mut TankSnapshot,
        command: Command<TankRequest, Infallible, Infallible, Infallible>,
        now: Now,
    ) -> Decision<Self> {
        let Command::Request(TankRequest::SetLevel(level)) = command;
        if level > snapshot.capacity {
            return Outcome::reject(AboveCapacity {
                level,
                capacity: snapshot.capacity,
            });
        }
        snapshot.level = level;
        snapshot.level_set_at = now.wall;
        assert_ne!(level, LEVEL_THAT_PANICS_IN_HANDLE);
        Outcome::Applied {
            events: if level == 0 {
                vec![TankEvent::LevelChanged(level), TankEvent::Emptied]
            } else {
                vec![TankEvent::LevelChanged(level)]
            },
            effects: Vec::new(),
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: blueos_domain::IoError,
    ) -> Command<TankRequest, Infallible, Infallible, Infallible> {
        match request {}
    }
}

impl DomainQueries for Tank {
    type Query = TankQuery;
    type Response = TankResponse;

    fn query(snapshot: &TankSnapshot, query: TankQuery, _now: Now) -> TankResponse {
        match query {
            TankQuery::Level => TankResponse::Level(snapshot.level),
            TankQuery::Other => TankResponse::Other,
            TankQuery::Panic => panic!("the tank cannot answer"),
        }
    }
}

impl Service for TankService {
    type Domain = Tank;
    type Context = ();
    type Arguments = TankArguments;

    const NAME: &'static str = "tank";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TankArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<TankArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        let capacity = service.arguments().capacity;
        if capacity == 0 {
            return Err(ServiceError::Build(NoCapacity.into()));
        }
        Ok(with_tank_queries(
            ServiceBuilder::new(TankSnapshot::empty(capacity))
                .command("SetLevel", |request: SetLevelGoal| {
                    Ok(TankRequest::SetLevel(request.level))
                })
                .command("SetPercent", move |request: SetLevelGoal| {
                    if request.level > 100 {
                        return Err(NotAPercent(request.level).into());
                    }
                    let level = u16::from(capacity) * u16::from(request.level) / 100;
                    Ok(TankRequest::SetLevel(u8::try_from(level).unwrap()))
                }),
        )
        .state("level_set_at", |snapshot: &TankSnapshot| Time {
            sec: i32::try_from(snapshot.level_set_at.as_secs()).unwrap(),
            nanosec: snapshot.level_set_at.subsec_nanos(),
        })
        .state("tank", |snapshot: &TankSnapshot| {
            assert_ne!(snapshot.level, LEVEL_THAT_PANICS_IN_PROJECTION);
            PumpState {
                level: snapshot.level,
                max_level: snapshot.capacity,
                ..PumpState::default()
            }
        })
        .event("LevelChanged", |event: &TankEvent| match event {
            TankEvent::LevelChanged(level) => Some(LevelResponse {
                level: *level,
                max_level: 0,
            }),
            TankEvent::Emptied => None,
        })
        .event("Emptied", |event: &TankEvent| {
            matches!(event, TankEvent::Emptied).then(Empty::default)
        }))
    }
}

pub fn level_response(response: TankResponse) -> Option<LevelResponse> {
    match response {
        TankResponse::Level(level) => Some(LevelResponse {
            level,
            max_level: 0,
        }),
        TankResponse::Other => None,
    }
}

fn with_tank_queries(builder: ServiceBuilder<Tank>) -> ServiceBuilder<Tank> {
    builder
        .query(
            "Level",
            |_request: Empty| Ok(TankQuery::Level),
            level_response,
        )
        .query(
            "Other",
            |_request: Empty| Ok(TankQuery::Other),
            level_response,
        )
        .query(
            "Panics",
            |_request: Empty| Ok(TankQuery::Panic),
            level_response,
        )
        .query(
            "Refused",
            |request: SetLevelGoal| -> Result<TankQuery, Refusal> {
                Err(NotAPercent(request.level).into())
            },
            level_response,
        )
}
