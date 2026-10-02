//! Kernel Effects: ordered IO, typed timers, and synchronous Effect rollback.

use core::{
    convert::Infallible,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};
use tokio::time::{advance, timeout};

use blueos_api::{Message, cdr_encoding, command_key};
use blueos_domain::{Command, Decision, Domain, DomainQueries, Effect, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

static PANIC_GUARD: AtomicUsize = AtomicUsize::new(0);
static BLOCKING_RELEASE: AtomicBool = AtomicBool::new(false);

struct EffectsService;

#[derive(clap::Args)]
struct EffectsArguments {
    #[arg(long, default_value_t = 10)]
    capacity: u8,
}

#[derive(Clone)]
struct EffectsContext {
    expected_capacity: u8,
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
        Ok(ServiceBuilder::new(EffectsSnapshot {
            level: 0,
            capacity,
            failed_io_requests: 0,
            succeeded_io_requests: 0,
            last_failed_request: None,
            tick_count: 0,
            blocking_io_running: false,
        })
        .context(EffectsContext {
            expected_capacity: capacity,
        })
        .blocking_io(|_io_context, _snapshot, request| match request {
            EffectsIoRequest::BlockingHold => {
                while !BLOCKING_RELEASE.load(Ordering::SeqCst) {
                    std::thread::yield_now();
                }
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

async fn io_counts(harness: &Harness<EffectsService>) -> LevelQueryResponse {
    harness.state::<LevelQueryResponse>("io").await
}

async fn drain_io(harness: &Harness<EffectsService>) {
    for _ in 0..32 {
        tokio::task::yield_now().await;
        let counts = io_counts(harness).await;
        if counts.level >= 1 && counts.max_level >= 1 {
            return;
        }
    }
    panic!("the IO chain did not finish");
}

#[tokio::test(start_paused = true)]
async fn failed_first_io_still_runs_second() {
    let harness = Harness::<EffectsService>::start(EffectsArguments { capacity: 10 })
        .await
        .unwrap();
    let ack = harness.send("RunIoChain", &EmptyRequest::default()).await;
    assert!(ack.accepted);
    drain_io(&harness).await;
    let counts = io_counts(&harness).await;
    assert_eq!(counts.level, 1);
    assert_eq!(counts.max_level, 1);
}

#[tokio::test(start_paused = true)]
async fn io_panic_reaches_domain_as_io_failed() {
    let harness = Harness::<EffectsService>::start(EffectsArguments { capacity: 10 })
        .await
        .unwrap();
    let ack = harness.send("RunIoPanic", &EmptyRequest::default()).await;
    assert!(ack.accepted);
    for _ in 0..32 {
        tokio::task::yield_now().await;
        if io_counts(&harness).await.level >= 1 {
            break;
        }
    }
    let counts = io_counts(&harness).await;
    assert_eq!(counts.level, 1);
    assert_eq!(counts.max_level, 0);
}

#[tokio::test(start_paused = true)]
async fn rearmed_timer_fires_once_at_new_time() {
    let harness = Harness::<EffectsService>::start(EffectsArguments { capacity: 10 })
        .await
        .unwrap();
    harness.send("ArmTimer", &EmptyRequest::default()).await;
    harness.send("ReArmTimer", &EmptyRequest::default()).await;
    advance(Duration::from_secs(5)).await;
    let observed = timeout(Duration::from_secs(1), async {
        loop {
            if harness.state::<LevelQueryResponse>("ticks").await.level == 1 {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    assert!(observed.is_ok());
    advance(Duration::from_secs(10)).await;
    tokio::task::yield_now().await;
    assert_eq!(harness.state::<LevelQueryResponse>("ticks").await.level, 1);
}

#[tokio::test(start_paused = true)]
async fn cancelled_timer_never_fires() {
    let harness = Harness::<EffectsService>::start(EffectsArguments { capacity: 10 })
        .await
        .unwrap();
    harness.send("CancelTimer", &EmptyRequest::default()).await;
    advance(Duration::from_secs(20)).await;
    let observed = timeout(Duration::from_millis(1), async {
        harness.state::<LevelQueryResponse>("ticks").await.level
    })
    .await;
    assert_eq!(observed.unwrap(), 0);
}

#[tokio::test(start_paused = true)]
async fn synchronous_io_effect_failure_rolls_back_command() {
    let harness = Harness::<EffectsWithoutIoService>::start(EffectsArguments { capacity: 10 })
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
    let harness = Harness::<EffectsService>::start(EffectsArguments { capacity: 10 })
        .await
        .unwrap();
    let ack = harness
        .send("RecordCapacity", &EmptyRequest::default())
        .await;
    assert!(ack.accepted);
    for _ in 0..8 {
        tokio::task::yield_now().await;
    }
}

#[tokio::test(start_paused = true)]
async fn query_answers_while_blocking_io_is_held() {
    BLOCKING_RELEASE.store(false, Ordering::SeqCst);
    let harness = Harness::<EffectsService>::start(EffectsArguments { capacity: 10 })
        .await
        .unwrap();
    let backend = std::sync::Arc::clone(harness.backend());
    let send = tokio::spawn(async move {
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
    let observed = timeout(Duration::from_secs(1), async {
        loop {
            if harness
                .query::<EmptyRequest, LevelQueryResponse>(
                    "blocking_active",
                    &EmptyRequest::default(),
                )
                .await
                .level
                == 1
            {
                return harness
                    .query::<EmptyRequest, LevelQueryResponse>(
                        "blocking_active",
                        &EmptyRequest::default(),
                    )
                    .await
                    .level;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("blocking IO should start");
    assert_eq!(observed, 1);
    BLOCKING_RELEASE.store(true, Ordering::SeqCst);
    send.await.expect("the Command should finish");
    let cleared = timeout(Duration::from_secs(1), async {
        loop {
            if harness
                .query::<EmptyRequest, LevelQueryResponse>(
                    "blocking_active",
                    &EmptyRequest::default(),
                )
                .await
                .level
                == 0
            {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    cleared.expect("blocking IO should finish after release");
}

#[tokio::test(start_paused = true)]
async fn effect_recorder_sees_effects_without_running_them() {
    let (harness, log) =
        Harness::<EffectsService>::start_recording_effects(EffectsArguments { capacity: 10 })
            .await
            .unwrap();
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
    let observed = timeout(Duration::from_millis(1), async {
        harness.state::<LevelQueryResponse>("ticks").await.level
    })
    .await;
    assert_eq!(observed.unwrap(), 0);
}
