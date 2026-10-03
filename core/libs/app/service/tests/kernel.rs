//! The Kernel as a client sees it: a test Service's real `build` on the channel backend, with a paused clock.

use core::{
    convert::Infallible,
    mem,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::sync::{Arc, Mutex, PoisonError};

use futures_util::{future::BoxFuture, stream};
use tokio::{sync::Semaphore, task::JoinSet, time::timeout};

use blueos_api::{
    CommandAck, Message, cdr_encoding, command_key, event_key, job_result_key, jobs_key, query_key,
    state_key,
};
use blueos_comms::{
    CommsBackend, CommsError, LivelinessSubscriber, LivelinessToken, Query, QueryBody, Queryable,
    Reply, ReplyError, Sample, Subscriber, channel::ChannelBackend,
};
use blueos_domain::{Command, Decision, Domain, DomainQueries, Now, Outcome};
use blueos_idl::{
    Error as IdlError,
    cdr::{Reader, Writer},
    message::CdrStruct,
    msg::{
        blueos_example_msgs::{EmptyRequest, LevelQueryResponse, PumpState, SetLevelRequest},
        blueos_msgs::{CommandAckStatus, JobStatusStatus},
        builtin_interfaces::Time,
    },
};
use blueos_jobs::JobId;
use blueos_service::{
    Kernel, Refusal, Service, ServiceBuilder, ServiceContext, ServiceError,
    testing::{Harness, PausedClock, WALL_CLOCK_AT_START},
};

/// `handle` panics after it changed the Snapshot.
const LEVEL_THAT_PANICS_IN_HANDLE: u8 = 99;
/// The `tank` State's projection panics.
const LEVEL_THAT_PANICS_IN_PROJECTION: u8 = 98;
/// The `FragileLevel` State fails to encode.
const LEVEL_THAT_FAILS_TO_ENCODE: u8 = 13;

/// A Service whose one Command sets the level of a tank.
struct TankService;

#[derive(clap::Args)]
struct TankArguments {
    #[arg(long, default_value_t = 100)]
    capacity: u8,
}

struct Tank;

#[derive(Clone)]
struct TankSnapshot {
    level: u8,
    capacity: u8,
    /// The wall-clock time of the last applied `SetLevel`.
    level_set_at: Duration,
}

enum TankRequest {
    SetLevel(u8),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TankTimerKey {}

enum TankEvent {
    LevelChanged(u8),
    Emptied,
}

enum TankQuery {
    Level,
    /// Answered with a Response that the `Level` conversion does not publish.
    Other,
    Panic,
}

enum TankResponse {
    Level(u8),
    Other,
}

#[derive(Debug, thiserror::Error)]
#[error("level {level} is above the capacity {capacity}")]
struct AboveCapacity {
    level: u8,
    capacity: u8,
}

#[derive(Debug, thiserror::Error)]
#[error("a tank needs a capacity above 0")]
struct NoCapacity;

#[derive(Debug, thiserror::Error)]
#[error("{0} is not a percentage")]
struct NotAPercent(u8);

/// The tank, with IO queries that read a simulated sensor outside the Inbox.
struct ProbeService;

#[derive(clap::Args, Default)]
struct ProbeArguments {
    #[arg(skip)]
    sensor: Sensor,
}

/// A simulated level sensor: each permit lets one `Probe` read it.
#[derive(Clone)]
struct Sensor(Arc<Semaphore>);

/// The tank, publishing a State that fails to encode at one level.
struct FragileTankService;

/// A State Message whose encoding fails at [`LEVEL_THAT_FAILS_TO_ENCODE`].
#[derive(Debug)]
struct FragileLevel {
    level: u8,
}

/// The tank, with a Command name that is not a valid key.
struct MisnamedTankService;

/// A backbone that closes every endpoint as soon as it is declared.
struct ClosedBackend;

/// The channel backend, recording every publish and every reply in order, and failing publishes on demand.
#[derive(Default)]
struct RecordingBackend {
    bus: ChannelBackend,
    journal: Arc<Mutex<Vec<String>>>,
    publishes_fail: AtomicBool,
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
        Ok(ServiceBuilder::new(TankSnapshot::empty(capacity))
            .command("SetLevel", |request: SetLevelRequest| {
                Ok(TankRequest::SetLevel(request.level))
            })
            .command("SetPercent", move |request: SetLevelRequest| {
                if request.level > 100 {
                    return Err(NotAPercent(request.level).into());
                }
                let level = u16::from(capacity) * u16::from(request.level) / 100;
                Ok(TankRequest::SetLevel(u8::try_from(level).unwrap()))
            })
            .query(
                "Level",
                |_request: EmptyRequest| Ok(TankQuery::Level),
                level_response,
            )
            .query(
                "Other",
                |_request: EmptyRequest| Ok(TankQuery::Other),
                level_response,
            )
            .query(
                "Panics",
                |_request: EmptyRequest| Ok(TankQuery::Panic),
                level_response,
            )
            .query(
                "Refused",
                |request: SetLevelRequest| -> Result<TankQuery, Refusal> {
                    Err(NotAPercent(request.level).into())
                },
                level_response,
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
                TankEvent::LevelChanged(level) => Some(LevelQueryResponse {
                    level: *level,
                    max_level: 0,
                }),
                TankEvent::Emptied => None,
            })
            .event("Emptied", |event: &TankEvent| {
                matches!(event, TankEvent::Emptied).then(EmptyRequest::default)
            }))
    }
}

impl Service for FragileTankService {
    type Domain = Tank;
    type Context = ();
    type Arguments = TankArguments;

    const NAME: &'static str = "fragile_tank";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TankArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<TankArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        Ok(
            ServiceBuilder::new(TankSnapshot::empty(service.arguments().capacity))
                .command("SetLevel", |request: SetLevelRequest| {
                    Ok(TankRequest::SetLevel(request.level))
                })
                .query(
                    "Level",
                    |_request: EmptyRequest| Ok(TankQuery::Level),
                    |response: TankResponse| match response {
                        TankResponse::Level(level) => Some(FragileLevel { level }),
                        TankResponse::Other => None,
                    },
                )
                .state("tank", |snapshot: &TankSnapshot| FragileLevel {
                    level: snapshot.level,
                }),
        )
    }
}

impl Service for ProbeService {
    type Domain = Tank;
    type Context = ();
    type Arguments = ProbeArguments;

    const NAME: &'static str = "probe";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<ProbeArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<ProbeArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        Ok(ServiceBuilder::new(TankSnapshot::empty(100))
            .command("SetLevel", |request: SetLevelRequest| {
                Ok(TankRequest::SetLevel(request.level))
            })
            .io_query("Probe", {
                let sensor = Arc::clone(&service.arguments().sensor.0);
                move |request: SetLevelRequest| {
                    let sensor = Arc::clone(&sensor);
                    Box::pin(async move {
                        sensor.acquire().await?.forget();
                        assert_ne!(request.level, LEVEL_THAT_PANICS_IN_HANDLE);
                        if request.level > 100 {
                            return Err(NotAPercent(request.level).into());
                        }
                        Ok(LevelQueryResponse {
                            level: request.level,
                            max_level: 100,
                        })
                    })
                }
            })
            .io_query("FragileProbe", |_request: EmptyRequest| {
                Box::pin(async {
                    Ok(FragileLevel {
                        level: LEVEL_THAT_FAILS_TO_ENCODE,
                    })
                })
            }))
    }
}

impl Service for MisnamedTankService {
    type Domain = Tank;
    type Context = ();
    type Arguments = TankArguments;

    const NAME: &'static str = "misnamed_tank";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TankArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<TankArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        Ok(
            ServiceBuilder::new(TankSnapshot::empty(service.arguments().capacity))
                .command("Set#Level", |request: SetLevelRequest| {
                    Ok(TankRequest::SetLevel(request.level))
                }),
        )
    }
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
    fn empty(capacity: u8) -> Self {
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

impl Default for Sensor {
    fn default() -> Self {
        Self(Arc::new(Semaphore::new(0)))
    }
}

impl CommsBackend for ClosedBackend {
    fn publish(&self, _sample: Sample) -> BoxFuture<'_, Result<(), CommsError>> {
        Box::pin(async { Ok(()) })
    }

    fn subscribe<'a>(
        &'a self,
        _key_expression: &'a str,
    ) -> BoxFuture<'a, Result<Subscriber, CommsError>> {
        Box::pin(async { Ok(Subscriber::new(stream::empty())) })
    }

    fn declare_queryable<'a>(
        &'a self,
        _key: &'a str,
    ) -> BoxFuture<'a, Result<Queryable, CommsError>> {
        Box::pin(async { Ok(Queryable::new(stream::empty())) })
    }

    fn get<'a>(
        &'a self,
        _key_expression: &'a str,
        _body: Option<QueryBody>,
        _timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<Reply>, CommsError>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn declare_liveliness<'a>(
        &'a self,
        _key: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessToken, CommsError>> {
        Box::pin(async { Ok(LivelinessToken::new(|| ())) })
    }

    fn subscribe_liveliness<'a>(
        &'a self,
        _key_expression: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessSubscriber, CommsError>> {
        Box::pin(async { Ok(LivelinessSubscriber::new(stream::empty())) })
    }

    fn get_liveliness<'a>(
        &'a self,
        _key_expression: &'a str,
        _timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<String>, CommsError>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}

impl CommsBackend for RecordingBackend {
    fn publish(&self, sample: Sample) -> BoxFuture<'_, Result<(), CommsError>> {
        if self.publishes_fail.load(Ordering::SeqCst) {
            return Box::pin(async {
                Err(CommsError::backend(std::io::Error::from(
                    std::io::ErrorKind::NotConnected,
                )))
            });
        }
        record(&self.journal, format!("publish {}", sample.key()));
        self.bus.publish(sample)
    }

    fn subscribe<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<Subscriber, CommsError>> {
        self.bus.subscribe(key_expression)
    }

    fn declare_queryable<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<Queryable, CommsError>> {
        Box::pin(async move {
            let queries = stream::unfold(self.bus.declare_queryable(key).await?, {
                let journal = Arc::clone(&self.journal);
                let key = key.to_owned();
                move |mut queryable| {
                    let journal = Arc::clone(&journal);
                    let key = key.clone();
                    async move {
                        let query = queryable.recv().await?;
                        Some((recorded(query, key, journal), queryable))
                    }
                }
            });
            Ok(Queryable::new(queries))
        })
    }

    fn get<'a>(
        &'a self,
        key_expression: &'a str,
        body: Option<QueryBody>,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<Reply>, CommsError>> {
        self.bus.get(key_expression, body, timeout)
    }

    fn declare_liveliness<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessToken, CommsError>> {
        self.bus.declare_liveliness(key)
    }

    fn subscribe_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessSubscriber, CommsError>> {
        self.bus.subscribe_liveliness(key_expression)
    }

    fn get_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<String>, CommsError>> {
        self.bus.get_liveliness(key_expression, timeout)
    }
}

impl RecordingBackend {
    /// Takes what was recorded so far.
    fn take_journal(&self) -> Vec<String> {
        mem::take(&mut self.journal.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

#[tokio::test(start_paused = true)]
async fn a_client_reads_the_new_state_right_after_the_ack() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    assert_eq!(harness.state::<PumpState>("tank").await.level, 0);

    let ack = harness
        .send("SetLevel", &SetLevelRequest { level: 42 })
        .await;

    assert!(ack.accepted, "{}", ack.reason);
    let state = harness.state::<PumpState>("tank").await;
    assert_eq!((state.level, state.max_level), (42, 100));
}

#[tokio::test(start_paused = true)]
async fn a_rejected_request_leaves_the_state_and_tells_the_client_why() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 10 })
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelRequest { level: 4 })
        .await;

    let ack = harness
        .send("SetLevel", &SetLevelRequest { level: 11 })
        .await;

    assert!(!ack.accepted);
    assert_eq!(ack.reason, "level 11 is above the capacity 10");
    assert_eq!(ack.status, CommandAckStatus::StatusUnknown);
    assert_eq!(harness.state::<PumpState>("tank").await.level, 4);
}

#[tokio::test(start_paused = true)]
async fn states_are_published_before_the_ack_and_events_after_it() {
    let backend = Arc::new(RecordingBackend::default());
    let harness = Harness::<TankService>::start_on(
        Arc::<RecordingBackend>::clone(&backend),
        TankArguments { capacity: 100 },
    )
    .await
    .unwrap();
    backend.take_journal();

    harness
        .send("SetLevel", &SetLevelRequest { level: 3 })
        .await;

    assert_eq!(
        backend.take_journal(),
        [
            format!("publish {}", state_key(TankService::NAME, "level_set_at")),
            format!("publish {}", state_key(TankService::NAME, "tank")),
            format!("publish {}", jobs_key(TankService::NAME)),
            format!("reply {}", command_key(TankService::NAME, "SetLevel")),
            format!("publish {}", event_key(TankService::NAME, "LevelChanged")),
            format!("publish {}", job_result_key(TankService::NAME, "SetLevel")),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn send_awaiting_ack_returns_rejection_when_handle_panics() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelRequest { level: 5 })
        .await;
    let ack = harness
        .command_sender()
        .send_awaiting_ack(Command::Request(TankRequest::SetLevel(
            LEVEL_THAT_PANICS_IN_HANDLE,
        )))
        .await
        .expect("the Inbox stays open");
    assert!(!ack.accepted);
    assert_eq!(ack.reason, "the Command panicked, so nothing changed");
    assert_eq!(harness.state::<PumpState>("tank").await.level, 5);
}

#[tokio::test(start_paused = true)]
async fn a_panic_in_handle_restores_the_snapshot_and_rejects_the_ack() {
    assert_a_panic_changes_nothing(LEVEL_THAT_PANICS_IN_HANDLE).await;
}

#[tokio::test(start_paused = true)]
async fn a_panic_in_a_projection_restores_the_snapshot_and_drops_the_domain_events() {
    assert_a_panic_changes_nothing(LEVEL_THAT_PANICS_IN_PROJECTION).await;
}

async fn assert_a_panic_changes_nothing(level_that_panics: u8) {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelRequest { level: 5 })
        .await;
    let mut events = harness
        .backend()
        .subscribe(&event_key(TankService::NAME, "LevelChanged"))
        .await
        .unwrap();

    let ack = harness
        .send(
            "SetLevel",
            &SetLevelRequest {
                level: level_that_panics,
            },
        )
        .await;

    assert!(!ack.accepted);
    assert_eq!(ack.reason, "the Command panicked, so nothing changed");
    assert_eq!(harness.state::<PumpState>("tank").await.level, 5);
    harness
        .send("SetLevel", &SetLevelRequest { level: 6 })
        .await;
    let first_event = next_sample(&mut events).await;
    let first_event = LevelQueryResponse::decode(&first_event.payload().to_bytes()).unwrap();
    assert_eq!(first_event.level, 6);
}

#[tokio::test(start_paused = true)]
async fn a_failed_publish_does_not_stop_the_inbox_and_the_state_is_sent_on_the_next_command() {
    let backend = Arc::new(RecordingBackend::default());
    let harness = Harness::<TankService>::start_on(
        Arc::<RecordingBackend>::clone(&backend),
        TankArguments { capacity: 100 },
    )
    .await
    .unwrap();
    let mut states = harness
        .backend()
        .subscribe(&state_key(TankService::NAME, "tank"))
        .await
        .unwrap();
    backend.publishes_fail.store(true, Ordering::SeqCst);

    let ack = harness
        .send("SetLevel", &SetLevelRequest { level: 1 })
        .await;
    backend.publishes_fail.store(false, Ordering::SeqCst);
    // The same State twice: sent by the first, because the failed publish was never stored, and deduplicated by
    // the second.
    for level in [1, 1, 2] {
        harness.send("SetLevel", &SetLevelRequest { level }).await;
    }

    assert!(ack.accepted, "{}", ack.reason);
    let mut published_levels = Vec::new();
    for _sample in 0..2 {
        let sample = next_sample(&mut states).await;
        published_levels.push(
            PumpState::decode(&sample.payload().to_bytes())
                .unwrap()
                .level,
        );
    }
    assert_eq!(published_levels, [1, 2]);
}

#[tokio::test(start_paused = true)]
async fn a_state_that_fails_to_encode_does_not_stop_the_inbox() {
    let harness = Harness::<FragileTankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();

    let failing_ack = harness
        .send(
            "SetLevel",
            &SetLevelRequest {
                level: LEVEL_THAT_FAILS_TO_ENCODE,
            },
        )
        .await;
    let next_ack = harness
        .send("SetLevel", &SetLevelRequest { level: 2 })
        .await;

    assert!(failing_ack.accepted, "{}", failing_ack.reason);
    assert!(next_ack.accepted, "{}", next_ack.reason);
    assert_eq!(harness.state::<FragileLevel>("tank").await.level, 2);
}

/// The next sample, or a failed test when none comes. Time is paused, so the timeout costs no real time.
async fn next_sample(subscriber: &mut Subscriber) -> Sample {
    timeout(Duration::from_secs(10), subscriber.recv())
        .await
        .expect("a sample is published")
        .expect("the subscription is open")
}

#[tokio::test(start_paused = true)]
async fn handle_receives_the_time_of_the_injected_clock() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    tokio::time::advance(Duration::from_millis(5_250)).await;

    harness
        .send("SetLevel", &SetLevelRequest { level: 7 })
        .await;

    let level_set_at = harness.state::<Time>("level_set_at").await;
    let expected = WALL_CLOCK_AT_START + Duration::from_millis(5_250);
    assert_eq!(
        (level_set_at.sec, level_set_at.nanosec),
        (i32::try_from(expected.as_secs()).unwrap(), 250_000_000)
    );
}

#[tokio::test(start_paused = true)]
async fn a_request_that_does_not_decode_is_rejected_before_the_domain() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    let body = QueryBody::new(vec![0xFF], cdr_encoding(SetLevelRequest::SCHEMA_NAME))
        .with_attachment(JobId::from_u128(1).to_string().into_bytes());

    let replies = harness
        .backend()
        .get(
            &command_key(TankService::NAME, "SetLevel"),
            Some(body),
            Duration::from_secs(10),
        )
        .await
        .unwrap();

    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one ack, got {replies:?}");
    };
    let ack = CommandAck::decode(&reply.payload().to_bytes()).unwrap();
    assert!(!ack.accepted);
    assert_eq!(
        ack.reason,
        "the Request does not decode: invalid CDR encapsulation header"
    );
    assert_eq!(harness.state::<PumpState>("tank").await.level, 0);
}

#[tokio::test(start_paused = true)]
async fn a_build_that_refuses_its_arguments_stops_startup() {
    let started = Harness::<TankService>::start(TankArguments { capacity: 0 }).await;

    let Err(ServiceError::Build(reason)) = started else {
        panic!("a tank with no capacity must not start");
    };
    assert!(reason.downcast_ref::<NoCapacity>().is_some());
}

#[tokio::test(start_paused = true)]
async fn an_endpoint_the_backbone_refuses_stops_startup() {
    let started = Harness::<MisnamedTankService>::start(TankArguments { capacity: 100 }).await;

    let Err(ServiceError::DeclareEndpoint { key, .. }) = started else {
        panic!("a Service with an invalid key must not start");
    };
    assert_eq!(key, command_key(MisnamedTankService::NAME, "Set#Level"));
}

#[tokio::test(start_paused = true)]
async fn a_state_that_never_reached_the_backbone_has_no_value_to_read() {
    let backend = Arc::new(RecordingBackend::default());
    backend.publishes_fail.store(true, Ordering::SeqCst);
    let harness = Harness::<TankService>::start_on(
        Arc::<RecordingBackend>::clone(&backend),
        TankArguments { capacity: 100 },
    )
    .await
    .unwrap();

    let replies = harness
        .backend()
        .get(
            &state_key(TankService::NAME, "tank"),
            None,
            Duration::from_secs(10),
        )
        .await
        .unwrap();

    assert!(replies.is_empty(), "{replies:?}");
}

#[tokio::test(start_paused = true)]
async fn each_event_endpoint_publishes_only_the_domain_events_it_selects() {
    let backend = Arc::new(RecordingBackend::default());
    let harness = Harness::<TankService>::start_on(
        Arc::<RecordingBackend>::clone(&backend),
        TankArguments { capacity: 100 },
    )
    .await
    .unwrap();
    harness
        .send("SetLevel", &SetLevelRequest { level: 3 })
        .await;
    backend.take_journal();

    harness
        .send("SetLevel", &SetLevelRequest { level: 0 })
        .await;

    // `level_set_at` is not published again: the time did not move, so its value did not change.
    assert_eq!(
        backend.take_journal(),
        [
            format!("publish {}", state_key(TankService::NAME, "tank")),
            format!("publish {}", jobs_key(TankService::NAME)),
            format!("reply {}", command_key(TankService::NAME, "SetLevel")),
            format!("publish {}", event_key(TankService::NAME, "LevelChanged")),
            format!("publish {}", event_key(TankService::NAME, "Emptied")),
            format!("publish {}", job_result_key(TankService::NAME, "SetLevel")),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn a_kernel_with_only_io_queries_keeps_answering_them() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let builder = ServiceBuilder::<Tank>::new(TankSnapshot::empty(100)).io_query(
        "Probe",
        |request: SetLevelRequest| {
            Box::pin(async move {
                Ok(LevelQueryResponse {
                    level: request.level,
                    max_level: 100,
                })
            })
        },
    );
    let kernel = Kernel::start(
        "sensor",
        builder,
        (),
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    )
    .await
    .unwrap();
    let mut running = JoinSet::new();
    running.spawn(kernel.run());
    // Lets `run` go as far as it can before the IO query arrives.
    tokio::task::yield_now().await;

    let body = QueryBody::new(
        SetLevelRequest { level: 4 }.encode().unwrap(),
        cdr_encoding(SetLevelRequest::SCHEMA_NAME),
    );
    let replies = backend
        .get(
            &query_key("sensor", "Probe"),
            Some(body),
            Duration::from_secs(10),
        )
        .await
        .unwrap();

    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one answer, got {replies:?}");
    };
    assert_eq!(
        LevelQueryResponse::decode(&reply.payload().to_bytes())
            .unwrap()
            .level,
        4
    );
}

#[tokio::test(start_paused = true)]
async fn the_kernel_stops_once_the_backbone_closes_every_endpoint() {
    let builder = TankService::build(
        &ServiceContext::new(
            TankArguments { capacity: 100 },
            blueos_service::testing::channel_session(),
        ),
        &(),
    )
    .unwrap()
    .io_query("Probe", |_request: EmptyRequest| {
        Box::pin(async { Ok(EmptyRequest::default()) })
    });
    let kernel = Kernel::start(
        TankService::NAME,
        builder,
        (),
        Arc::new(ClosedBackend),
        Arc::new(PausedClock::start()),
    )
    .await
    .unwrap();

    timeout(Duration::from_secs(10), kernel.run())
        .await
        .expect("the Kernel stops");
}

#[tokio::test(start_paused = true)]
#[should_panic(expected = "expected one ack from \"Missing\"")]
async fn the_harness_panics_when_a_command_gets_no_ack() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();

    harness.send("Missing", &SetLevelRequest { level: 1 }).await;
}

#[tokio::test(start_paused = true)]
#[should_panic(expected = "expected one value of \"missing\"")]
async fn the_harness_panics_when_a_state_has_no_value() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();

    harness.state::<PumpState>("missing").await;
}

#[tokio::test(start_paused = true)]
async fn a_domain_without_jobs_still_lists_the_instant_jobs_it_ran() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();

    let ack = harness
        .send("SetLevel", &SetLevelRequest { level: 42 })
        .await;

    assert_eq!(ack.status, CommandAckStatus::Succeeded);
    let jobs = harness.jobs().await;
    let [job] = jobs.jobs.as_slice() else {
        panic!("expected one Job, got {jobs:?}");
    };
    assert_eq!(
        (job.job_id.as_str(), job.job_type.as_str(), job.status),
        (ack.job_id.as_str(), "SetLevel", JobStatusStatus::Succeeded)
    );
}

#[tokio::test(start_paused = true)]
async fn a_request_its_conversion_refuses_is_rejected_before_the_domain() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 10 })
        .await
        .unwrap();

    let refused = harness
        .send("SetPercent", &SetLevelRequest { level: 150 })
        .await;
    let applied = harness
        .send("SetPercent", &SetLevelRequest { level: 50 })
        .await;

    assert!(!refused.accepted);
    assert_eq!(refused.reason, "150 is not a percentage");
    assert!(applied.accepted, "{}", applied.reason);
    assert_eq!(harness.state::<PumpState>("tank").await.level, 5);
}

#[tokio::test(start_paused = true)]
async fn a_query_is_answered_from_the_snapshot_left_by_the_commands_before_it() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelRequest { level: 42 })
        .await;

    let answer = harness
        .query::<_, LevelQueryResponse>("Level", &EmptyRequest::default())
        .await
        .unwrap();

    assert_eq!(answer.level, 42);
}

#[tokio::test(start_paused = true)]
async fn a_query_without_an_answer_replies_why_and_the_inbox_goes_on() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    let empty = EmptyRequest::default();

    let other_response = harness
        .query::<_, LevelQueryResponse>("Other", &empty)
        .await;
    let panicked = harness
        .query::<_, LevelQueryResponse>("Panics", &empty)
        .await;
    let refused = harness
        .query::<_, LevelQueryResponse>("Refused", &SetLevelRequest { level: 7 })
        .await;
    let undecodable = raw_query(&harness, TankService::NAME, "Level").await;

    assert_eq!(
        reason(other_response),
        "the Domain's Response does not belong to this Query"
    );
    assert_eq!(reason(panicked), "the Query panicked");
    assert_eq!(reason(refused), "7 is not a percentage");
    assert_eq!(
        reason(undecodable),
        "the Query does not decode: invalid CDR encapsulation header"
    );
    let answer = harness
        .query::<_, LevelQueryResponse>("Level", &empty)
        .await;
    assert_eq!(answer.unwrap().level, 0);
}

#[tokio::test(start_paused = true)]
async fn a_query_reply_that_fails_to_encode_replies_why() {
    let harness = Harness::<FragileTankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    harness
        .send(
            "SetLevel",
            &SetLevelRequest {
                level: LEVEL_THAT_FAILS_TO_ENCODE,
            },
        )
        .await;

    let answer = harness
        .query::<_, FragileLevel>("Level", &EmptyRequest::default())
        .await;

    assert_eq!(
        reason(answer),
        "the reply does not encode: invalid CDR length prefix"
    );
}

#[tokio::test(start_paused = true)]
async fn an_io_query_is_answered_outside_the_inbox() {
    let arguments = ProbeArguments::default();
    let sensor = arguments.sensor.clone();
    let harness = Harness::<ProbeService>::start(arguments).await.unwrap();

    // The sensor opens only after a Command was acknowledged, so a Probe that held the Inbox would never end.
    let (answer, ack) = tokio::join!(
        harness.query::<_, LevelQueryResponse>("Probe", &SetLevelRequest { level: 7 }),
        async {
            let ack = harness
                .send("SetLevel", &SetLevelRequest { level: 1 })
                .await;
            sensor.0.add_permits(1);
            ack
        }
    );

    assert!(ack.accepted, "{}", ack.reason);
    assert_eq!(answer.unwrap().level, 7);
}

#[tokio::test(start_paused = true)]
async fn an_io_query_without_an_answer_replies_why_and_the_next_one_is_answered() {
    let arguments = ProbeArguments::default();
    arguments.sensor.0.add_permits(3);
    let harness = Harness::<ProbeService>::start(arguments).await.unwrap();

    let refused = harness
        .query::<_, LevelQueryResponse>("Probe", &SetLevelRequest { level: 101 })
        .await;
    let panicked = harness
        .query::<_, LevelQueryResponse>(
            "Probe",
            &SetLevelRequest {
                level: LEVEL_THAT_PANICS_IN_HANDLE,
            },
        )
        .await;
    let undecodable = raw_query(&harness, ProbeService::NAME, "Probe").await;
    let unencodable = harness
        .query::<_, FragileLevel>("FragileProbe", &EmptyRequest::default())
        .await;
    let answer = harness
        .query::<_, LevelQueryResponse>("Probe", &SetLevelRequest { level: 3 })
        .await;

    assert_eq!(reason(refused), "101 is not a percentage");
    assert_eq!(reason(panicked), "the Query panicked");
    assert_eq!(
        reason(undecodable),
        "the Query does not decode: invalid CDR encapsulation header"
    );
    assert_eq!(
        reason(unencodable),
        "the reply does not encode: invalid CDR length prefix"
    );
    assert_eq!(answer.unwrap().level, 3);
}

#[tokio::test(start_paused = true)]
#[should_panic(expected = "expected one reply from \"Missing\"")]
async fn the_harness_panics_when_a_query_gets_no_reply() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();

    drop(
        harness
            .query::<_, LevelQueryResponse>("Missing", &EmptyRequest::default())
            .await,
    );
}

/// Sends a body that is not CDR to the query endpoint `name`.
async fn raw_query<S: Service>(
    harness: &Harness<S>,
    service: &str,
    name: &str,
) -> Result<Sample, ReplyError> {
    let body = QueryBody::new(vec![0xFF], cdr_encoding(EmptyRequest::SCHEMA_NAME));
    let replies = harness
        .backend()
        .get(
            &query_key(service, name),
            Some(body),
            Duration::from_secs(10),
        )
        .await
        .unwrap();
    let [reply] = replies.as_slice() else {
        panic!("expected one reply, got {replies:?}");
    };
    reply.clone()
}

/// The reason in an error reply.
fn reason<T: core::fmt::Debug>(answer: Result<T, ReplyError>) -> String {
    let error = answer.expect_err("the query has no answer");
    assert_eq!(error.encoding(), "text/plain");
    String::from_utf8(error.payload().to_bytes().into_owned()).unwrap()
}

fn level_response(response: TankResponse) -> Option<LevelQueryResponse> {
    match response {
        TankResponse::Level(level) => Some(LevelQueryResponse {
            level,
            max_level: 0,
        }),
        TankResponse::Other => None,
    }
}

fn record(journal: &Mutex<Vec<String>>, entry: String) {
    journal
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(entry);
}

/// Wraps `query` so that its reply is recorded when it is sent.
fn recorded(query: Query, key: String, journal: Arc<Mutex<Vec<String>>>) -> Query {
    let key_expression = query.key_expression().to_owned();
    let body = query.body().cloned();
    Query::new(key.clone(), key_expression, body, move |reply: Reply| {
        record(&journal, format!("reply {key}"));
        let sample = reply.expect("the Kernel never replies with an error");
        query.reply(sample.payload().clone(), sample.encoding().to_owned())
    })
}
