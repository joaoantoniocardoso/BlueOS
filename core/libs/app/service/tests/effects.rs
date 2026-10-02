//! Kernel Effects: ordered IO, typed timers, and synchronous Effect rollback.

use core::{
    convert::Infallible,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use std::sync::{Arc, Mutex, mpsc};

use tokio::time::{advance, timeout};

use blueos_api::{Message, cdr_encoding, command_key, state_key};
use blueos_comms::Subscriber;
use blueos_domain::{Command, Decision, Domain, DomainQueries, Effect, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

static PANIC_GUARD: AtomicUsize = AtomicUsize::new(0);

struct EffectsService;

/// Per-test gate for [`EffectsIoRequest::BlockingHold`]: no busy-wait and no process-wide statics.
struct BlockingHoldLatch {
    started: mpsc::SyncSender<()>,
    release: Mutex<mpsc::Receiver<()>>,
    io_applied: mpsc::SyncSender<()>,
}

#[derive(Clone, clap::Args)]
struct EffectsArguments {
    #[arg(long, default_value_t = 10)]
    capacity: u8,
    #[arg(skip)]
    blocking_hold: Option<Arc<BlockingHoldLatch>>,
    #[arg(skip)]
    record_capacity_done: Option<mpsc::SyncSender<()>>,
}

impl Default for EffectsArguments {
    fn default() -> Self {
        Self {
            capacity: 10,
            blocking_hold: None,
            record_capacity_done: None,
        }
    }
}

#[derive(Clone)]
struct EffectsContext {
    expected_capacity: u8,
    blocking_hold: Option<Arc<BlockingHoldLatch>>,
    record_capacity_done: Option<mpsc::SyncSender<()>>,
}

#[derive(Clone)]
struct EffectsSnapshot {
    level: u8,
    capacity: u8,
    failed_io_requests: u8,
    succeeded_io_requests: u8,
    last_failed_request: Option<EffectsIoRequest>,
    tick_count: u8,
    blocking_io_running: bool,
    blocking_io_applied: Option<mpsc::SyncSender<()>>,
}

enum EffectsRequest {
    RunIoChain,
    RunIoPanic,
    ArmTimer { after: Duration },
    ReArmTimer { after: Duration },
    CancelTimer,
    ScheduleIoWithoutExecutor,
    RecordCapacity,
    RunBlockingHold,
}

enum EffectsQuery {
    BlockingActive,
}

#[derive(Clone, Debug, PartialEq)]
enum EffectsIoRequest {
    Fail,
    Succeed,
    Panic,
    RecordCapacity,
    BlockingHold,
}

#[derive(Clone, Debug, PartialEq)]
enum EffectsIoResult {
    Failed {
        request: EffectsIoRequest,
        message: String,
    },
    Succeeded,
}

#[derive(Clone, Debug, PartialEq)]
enum EffectsTick {
    Fired,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum EffectsTimerKey {
    Alarm,
}

struct Effects;

impl Service for EffectsService {
    type Domain = Effects;
    type Context = EffectsContext;
    type Arguments = EffectsArguments;

    const NAME: &'static str = "effects";
    const VERSION: &'static str = "1.0.0";

    fn build(
        context: &ServiceContext<EffectsArguments>,
    ) -> Result<ServiceBuilder<Effects, Self::Context>, ServiceError> {
        let capacity = context.arguments().capacity;
        let blocking_io_applied = context
            .arguments()
            .blocking_hold
            .as_ref()
            .map(|latch| latch.io_applied.clone());
        Ok(ServiceBuilder::new(EffectsSnapshot {
            level: 0,
            capacity,
            failed_io_requests: 0,
            succeeded_io_requests: 0,
            last_failed_request: None,
            tick_count: 0,
            blocking_io_running: false,
            blocking_io_applied,
        })
        .context(EffectsContext {
            expected_capacity: capacity,
            blocking_hold: context.arguments().blocking_hold.clone(),
            record_capacity_done: context.arguments().record_capacity_done.clone(),
        })
        .blocking_io(|io_context, _snapshot, request| match request {
            EffectsIoRequest::BlockingHold => {
                let Some(latch) = &io_context.blocking_hold else {
                    return Err(IoError::new(
                        "BlockingHold requires a per-test BlockingHoldLatch",
                    ));
                };
                latch.started.send(()).map_err(|_| {
                    IoError::new("the test stopped waiting for blocking IO to start")
                })?;
                latch
                    .release
                    .lock()
                    .expect("the release mutex is not poisoned")
                    .recv()
                    .map_err(|_| IoError::new("the test stopped before releasing blocking IO"))?;
                Ok(Some(EffectsIoResult::Succeeded))
            }
            EffectsIoRequest::Fail
            | EffectsIoRequest::Succeed
            | EffectsIoRequest::Panic
            | EffectsIoRequest::RecordCapacity => Err(IoError::new("not a blocking IO request")),
        })
        .io(
            |io_context: &EffectsContext, snapshot: &EffectsSnapshot, request| {
                let expected_capacity = io_context.expected_capacity;
                let snapshot_capacity = snapshot.capacity;
                let record_capacity_done = io_context.record_capacity_done.clone();
                async move {
                    match request {
                        EffectsIoRequest::Fail => Err(IoError::new("the first IO step failed")),
                        EffectsIoRequest::Succeed => Ok(Some(EffectsIoResult::Succeeded)),
                        EffectsIoRequest::Panic => {
                            PANIC_GUARD.fetch_add(1, Ordering::SeqCst);
                            panic!("io panic test");
                        }
                        EffectsIoRequest::RecordCapacity => {
                            assert_eq!(expected_capacity, snapshot_capacity);
                            if let Some(done) = record_capacity_done {
                                let _ = done.send(());
                            }
                            Ok(None)
                        }
                        EffectsIoRequest::BlockingHold => {
                            Err(IoError::new("blocking IO belongs on a blocking thread"))
                        }
                    }
                }
            },
        )
        .command("RunIoChain", |_: EmptyRequest| EffectsRequest::RunIoChain)
        .command("RunIoPanic", |_: EmptyRequest| EffectsRequest::RunIoPanic)
        .command("ArmTimer", |_: EmptyRequest| EffectsRequest::ArmTimer {
            after: Duration::from_secs(10),
        })
        .command("ReArmTimer", |_: EmptyRequest| EffectsRequest::ReArmTimer {
            after: Duration::from_secs(5),
        })
        .command("CancelTimer", |_: EmptyRequest| EffectsRequest::CancelTimer)
        .command("ScheduleIoWithoutExecutor", |_: EmptyRequest| {
            EffectsRequest::ScheduleIoWithoutExecutor
        })
        .command("RecordCapacity", |_: EmptyRequest| {
            EffectsRequest::RecordCapacity
        })
        .command("RunBlockingHold", |_: EmptyRequest| {
            EffectsRequest::RunBlockingHold
        })
        .query(
            "blocking_active",
            |_: EmptyRequest| EffectsQuery::BlockingActive,
            |active: bool| LevelQueryResponse {
                level: u8::from(active),
                max_level: 0,
            },
        )
        .state("io", |snapshot: &EffectsSnapshot| LevelQueryResponse {
            level: snapshot.failed_io_requests,
            max_level: snapshot.succeeded_io_requests,
        })
        .state("ticks", |snapshot: &EffectsSnapshot| LevelQueryResponse {
            level: snapshot.tick_count,
            max_level: 0,
        }))
    }
}

impl Domain for Effects {
    type Snapshot = EffectsSnapshot;
    type Request = EffectsRequest;
    type IoResult = EffectsIoResult;
    type Tick = EffectsTick;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = EffectsIoRequest;
    type TimerKey = EffectsTimerKey;

    fn handle(
        snapshot: &mut Self::Snapshot,
        command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(EffectsRequest::RunIoChain) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![
                    Effect::Io(EffectsIoRequest::Fail),
                    Effect::Io(EffectsIoRequest::Succeed),
                ],
            },
            Command::Request(EffectsRequest::RunIoPanic) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Io(EffectsIoRequest::Panic)],
            },
            Command::Request(EffectsRequest::ArmTimer { after }) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Schedule {
                    after,
                    key: EffectsTimerKey::Alarm,
                    command: EffectsTick::Fired,
                }],
            },
            Command::Request(EffectsRequest::ReArmTimer { after }) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Schedule {
                    after,
                    key: EffectsTimerKey::Alarm,
                    command: EffectsTick::Fired,
                }],
            },
            Command::Request(EffectsRequest::CancelTimer) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![
                    Effect::Schedule {
                        after: Duration::from_secs(10),
                        key: EffectsTimerKey::Alarm,
                        command: EffectsTick::Fired,
                    },
                    Effect::Cancel(EffectsTimerKey::Alarm),
                ],
            },
            Command::Request(EffectsRequest::ScheduleIoWithoutExecutor) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Io(EffectsIoRequest::RecordCapacity)],
            },
            Command::Request(EffectsRequest::RecordCapacity) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Io(EffectsIoRequest::RecordCapacity)],
            },
            Command::Request(EffectsRequest::RunBlockingHold) => {
                snapshot.blocking_io_running = true;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: vec![Effect::Io(EffectsIoRequest::BlockingHold)],
                }
            }
            Command::IoResult(EffectsIoResult::Failed {
                request,
                message: _,
            }) => {
                snapshot.failed_io_requests += 1;
                snapshot.last_failed_request = Some(request);
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::IoResult(EffectsIoResult::Succeeded) => {
                snapshot.succeeded_io_requests += 1;
                snapshot.blocking_io_running = false;
                if let Some(applied) = snapshot.blocking_io_applied.take() {
                    let _ = applied.send(());
                }
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Tick(EffectsTick::Fired) => {
                snapshot.tick_count += 1;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::ObservedFact(never) => match never {},
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        Command::IoResult(EffectsIoResult::Failed {
            request,
            message: error.message().to_owned(),
        })
    }

    fn io_runs_on_blocking_thread(request: &Self::IoRequest) -> bool {
        matches!(request, EffectsIoRequest::BlockingHold)
    }
}

impl DomainQueries for Effects {
    type Query = EffectsQuery;
    type Response = bool;

    fn query(snapshot: &Self::Snapshot, query: Self::Query, _now: Now) -> Self::Response {
        match query {
            EffectsQuery::BlockingActive => snapshot.blocking_io_running,
        }
    }
}

struct EffectsWithoutIoService;

impl Service for EffectsWithoutIoService {
    type Domain = Effects;
    type Context = ();
    type Arguments = EffectsArguments;

    const NAME: &'static str = "effects";
    const VERSION: &'static str = "1.0.0";

    fn build(
        context: &ServiceContext<EffectsArguments>,
    ) -> Result<ServiceBuilder<Effects, Self::Context>, ServiceError> {
        let capacity = context.arguments().capacity;
        Ok(ServiceBuilder::new(EffectsSnapshot {
            level: 7,
            capacity,
            failed_io_requests: 0,
            succeeded_io_requests: 0,
            last_failed_request: None,
            tick_count: 0,
            blocking_io_running: false,
            blocking_io_applied: None,
        })
        .command("ScheduleIoWithoutExecutor", |_: EmptyRequest| {
            EffectsRequest::ScheduleIoWithoutExecutor
        })
        .state("level", |snapshot: &EffectsSnapshot| LevelQueryResponse {
            level: snapshot.level,
            max_level: 0,
        }))
    }
}

async fn subscribe_state(harness: &Harness<EffectsService>, name: &str) -> Subscriber {
    harness
        .backend()
        .subscribe(&state_key(EffectsService::NAME, name))
        .await
        .expect("the state key is valid")
}

async fn next_state_sample(subscriber: &mut Subscriber) -> LevelQueryResponse {
    let sample = timeout(Duration::from_secs(10), subscriber.recv())
        .await
        .expect("a state sample is published")
        .expect("the subscription is open");
    LevelQueryResponse::decode(&sample.payload().to_bytes()).expect("the state decodes")
}

async fn expect_no_state_sample(subscriber: &mut Subscriber) {
    let observed = timeout(Duration::from_millis(1), subscriber.recv()).await;
    assert!(
        observed.is_err(),
        "expected no further state publish on this subscription"
    );
}

#[tokio::test(start_paused = true)]
async fn failed_first_io_still_runs_second() {
    let harness = Harness::<EffectsService>::start(EffectsArguments::default())
        .await
        .unwrap();
    let mut io_states = subscribe_state(&harness, "io").await;
    let ack = harness.send("RunIoChain", &EmptyRequest::default()).await;
    assert!(ack.accepted);
    let after_fail = next_state_sample(&mut io_states).await;
    assert_eq!(after_fail.level, 1);
    assert_eq!(after_fail.max_level, 0);
    let after_success = next_state_sample(&mut io_states).await;
    assert_eq!(after_success.level, 1);
    assert_eq!(after_success.max_level, 1);
}

#[tokio::test(start_paused = true)]
async fn io_panic_reaches_domain_as_io_failed() {
    let harness = Harness::<EffectsService>::start(EffectsArguments::default())
        .await
        .unwrap();
    let mut io_states = subscribe_state(&harness, "io").await;
    let ack = harness.send("RunIoPanic", &EmptyRequest::default()).await;
    assert!(ack.accepted);
    let after_panic = next_state_sample(&mut io_states).await;
    assert_eq!(after_panic.level, 1);
    assert_eq!(after_panic.max_level, 0);
}

#[tokio::test(start_paused = true)]
async fn rearmed_timer_fires_once_at_new_time() {
    let harness = Harness::<EffectsService>::start(EffectsArguments::default())
        .await
        .unwrap();
    let mut tick_states = subscribe_state(&harness, "ticks").await;
    harness.send("ArmTimer", &EmptyRequest::default()).await;
    harness.send("ReArmTimer", &EmptyRequest::default()).await;
    advance(Duration::from_secs(5)).await;
    assert_eq!(next_state_sample(&mut tick_states).await.level, 1);
    advance(Duration::from_secs(10)).await;
    expect_no_state_sample(&mut tick_states).await;
}

#[tokio::test(start_paused = true)]
async fn cancelled_timer_never_fires() {
    let harness = Harness::<EffectsService>::start(EffectsArguments::default())
        .await
        .unwrap();
    let mut tick_states = subscribe_state(&harness, "ticks").await;
    harness.send("CancelTimer", &EmptyRequest::default()).await;
    advance(Duration::from_secs(20)).await;
    expect_no_state_sample(&mut tick_states).await;
}

#[tokio::test(start_paused = true)]
async fn synchronous_io_effect_failure_rolls_back_command() {
    let harness = Harness::<EffectsWithoutIoService>::start(EffectsArguments::default())
        .await
        .unwrap();
    let ack = harness
        .send("ScheduleIoWithoutExecutor", &EmptyRequest::default())
        .await;
    assert!(!ack.accepted);
    assert!(ack.reason.contains("IO executor"));
    assert_eq!(harness.state::<LevelQueryResponse>("level").await.level, 7);
}

#[tokio::test(start_paused = true)]
async fn io_executor_receives_context_and_snapshot() {
    let (done_sender, done_receiver) = mpsc::sync_channel(1);
    let wait_for_io = tokio::task::spawn_blocking(move || done_receiver.recv());
    let harness = Harness::<EffectsService>::start(EffectsArguments {
        capacity: 10,
        record_capacity_done: Some(done_sender),
        ..EffectsArguments::default()
    })
    .await
    .unwrap();
    let ack = harness
        .send("RecordCapacity", &EmptyRequest::default())
        .await;
    assert!(ack.accepted);
    wait_for_io
        .await
        .expect("join")
        .expect("RecordCapacity IO should run");
}

#[tokio::test(start_paused = true)]
async fn query_answers_while_blocking_io_is_held() {
    let (started_sender, started_receiver) = mpsc::sync_channel(0);
    let (release_sender, release_receiver) = mpsc::channel();
    let (io_applied_sender, io_applied_receiver) = mpsc::sync_channel(0);
    let latch = Arc::new(BlockingHoldLatch {
        started: started_sender,
        release: Mutex::new(release_receiver),
        io_applied: io_applied_sender,
    });
    let harness = Harness::<EffectsService>::start(EffectsArguments {
        capacity: 10,
        blocking_hold: Some(Arc::clone(&latch)),
        ..EffectsArguments::default()
    })
    .await
    .unwrap();
    let backend = Arc::clone(harness.backend());
    let command = tokio::spawn(async move {
        let body = blueos_comms::QueryBody::new(
            EmptyRequest::default().encode().unwrap(),
            cdr_encoding(EmptyRequest::SCHEMA_NAME),
        );
        backend
            .get(
                &command_key(EffectsService::NAME, "RunBlockingHold"),
                Some(body),
                Duration::from_secs(10),
            )
            .await
            .unwrap();
    });
    tokio::task::spawn_blocking(move || started_receiver.recv())
        .await
        .expect("started join")
        .expect("blocking IO should start");
    assert_eq!(
        harness
            .query::<EmptyRequest, LevelQueryResponse>(
                "blocking_active",
                &EmptyRequest::default(),
            )
            .await
            .level,
        1
    );
    release_sender
        .send(())
        .expect("the test should release blocking IO");
    tokio::task::spawn_blocking(move || io_applied_receiver.recv())
        .await
        .expect("io applied join")
        .expect("blocking IO result should reach the Domain");
    command.await.expect("the Command should finish");
    assert_eq!(
        harness
            .query::<EmptyRequest, LevelQueryResponse>(
                "blocking_active",
                &EmptyRequest::default(),
            )
            .await
            .level,
        0
    );
}

#[tokio::test(start_paused = true)]
async fn effect_recorder_sees_effects_without_running_them() {
    let (harness, log) =
        Harness::<EffectsService>::start_recording_effects(EffectsArguments::default())
            .await
            .unwrap();
    let mut tick_states = subscribe_state(&harness, "ticks").await;
    let ack = harness.send("CancelTimer", &EmptyRequest::default()).await;
    assert!(ack.accepted);
    let batch = log.last_batch().expect("one Command was applied");
    assert_eq!(
        batch,
        vec![
            Effect::Schedule {
                after: Duration::from_secs(10),
                key: EffectsTimerKey::Alarm,
                command: EffectsTick::Fired,
            },
            Effect::Cancel(EffectsTimerKey::Alarm),
        ]
    );
    advance(Duration::from_secs(20)).await;
    expect_no_state_sample(&mut tick_states).await;
}
