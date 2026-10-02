//! The Kernel as a client sees it: a test Service's real `build` on the channel backend, with a paused clock.

use core::{
    convert::Infallible,
    mem,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::sync::{Arc, Mutex, PoisonError};

use futures_util::{future::BoxFuture, stream};
use tokio::time::timeout;

use blueos_api::{
    CommandAck, JOB_ID_NONE, Message, cdr_encoding, command_key, event_key, state_key,
};
use blueos_comms::{
    CommsBackend, CommsError, Query, QueryBody, Queryable, Reply, Sample, Subscriber,
    channel::ChannelBackend,
};
use blueos_domain::{Command, Decision, Domain, Now, Outcome};
use blueos_idl::{
    Error as IdlError,
    cdr::{Reader, Writer},
    message::CdrStruct,
    msg::{
        blueos_example_msgs::{EmptyRequest, LevelQueryResponse, PumpState, SetLevelRequest},
        builtin_interfaces::Time,
    },
};
use blueos_service::{
    Kernel, Service, ServiceBuilder, ServiceContext, ServiceError,
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

enum TankEvent {
    LevelChanged(u8),
    Emptied,
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
    type Arguments = TankArguments;

    const NAME: &'static str = "tank";
    const VERSION: &'static str = "1.0.0";

    fn build(
        context: &ServiceContext<TankArguments>,
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        let capacity = context.arguments().capacity;
        if capacity == 0 {
            return Err(ServiceError::Build(NoCapacity.into()));
        }
        Ok(ServiceBuilder::new(TankSnapshot::empty(capacity))
            .command("SetLevel", |request: SetLevelRequest| {
                TankRequest::SetLevel(request.level)
            })
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
    type Arguments = TankArguments;

    const NAME: &'static str = "fragile_tank";
    const VERSION: &'static str = "1.0.0";

    fn build(
        context: &ServiceContext<TankArguments>,
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        Ok(
            ServiceBuilder::new(TankSnapshot::empty(context.arguments().capacity))
                .command("SetLevel", |request: SetLevelRequest| {
                    TankRequest::SetLevel(request.level)
                })
                .state("tank", |snapshot: &TankSnapshot| FragileLevel {
                    level: snapshot.level,
                }),
        )
    }
}

impl Service for MisnamedTankService {
    type Domain = Tank;
    type Arguments = TankArguments;

    const NAME: &'static str = "misnamed_tank";
    const VERSION: &'static str = "1.0.0";

    fn build(
        context: &ServiceContext<TankArguments>,
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        Ok(
            ServiceBuilder::new(TankSnapshot::empty(context.arguments().capacity))
                .command("Set#Level", |request: SetLevelRequest| {
                    TankRequest::SetLevel(request.level)
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
    type TimerKey = Infallible;

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
}

impl CommsBackend for RecordingBackend {
    fn publish(&self, sample: Sample) -> BoxFuture<'_, Result<(), CommsError>> {
        if self.publishes_fail.load(Ordering::SeqCst) {
            let key_expression = sample.key().to_owned();
            return Box::pin(async move {
                Err(CommsError::InvalidKeyExpression {
                    key_expression,
                    source: "the backbone is down".into(),
                })
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
    assert_eq!(ack.job_id, JOB_ID_NONE);
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
            format!("reply {}", command_key(TankService::NAME, "SetLevel")),
            format!("publish {}", event_key(TankService::NAME, "LevelChanged")),
        ]
    );
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
    let body = QueryBody::new(vec![0xFF], cdr_encoding(SetLevelRequest::SCHEMA_NAME));

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
            format!("reply {}", command_key(TankService::NAME, "SetLevel")),
            format!("publish {}", event_key(TankService::NAME, "LevelChanged")),
            format!("publish {}", event_key(TankService::NAME, "Emptied")),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn the_kernel_stops_once_the_backbone_closes_every_endpoint() {
    let builder =
        TankService::build(&ServiceContext::new(TankArguments { capacity: 100 })).unwrap();
    let kernel = Kernel::start(
        TankService::NAME,
        builder,
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
