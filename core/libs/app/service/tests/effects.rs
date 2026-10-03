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
use blueos_jobs::JobId;
use blueos_service::{
    Kernel, RunOutcome, Service, ServiceBuilder, ServiceContext, ServiceError,
    testing::{Harness, PausedClock, lock_unpoisoned},
};

static PANIC_GUARD: AtomicUsize = AtomicUsize::new(0);

struct EffectsService;

/// Per-test gate for [`EffectsIoRequest::BlockingHold`]: no busy-wait and no process-wide statics.
struct BlockingHoldLatch {
    started: mpsc::SyncSender<()>,
    release: Mutex<mpsc::Receiver<()>>,
    io_applied: mpsc::SyncSender<()>,
}

/// Per-test gate for async-held IO under a paused clock (shutdown drain tests).
struct AsyncHoldLatch {
    started: mpsc::SyncSender<()>,
    release: tokio::sync::Mutex<tokio::sync::mpsc::Receiver<()>>,
    io_applied: mpsc::SyncSender<()>,
}

#[derive(Clone, clap::Args)]
struct EffectsArguments {
    #[arg(long, default_value_t = 10)]
    capacity: u8,
    #[arg(skip)]
    blocking_hold: Option<Arc<BlockingHoldLatch>>,
    #[arg(skip)]
    async_hold: Option<Arc<AsyncHoldLatch>>,
    #[arg(skip)]
    record_capacity_done: Option<mpsc::SyncSender<()>>,
}

impl Default for EffectsArguments {
    fn default() -> Self {
        Self {
            capacity: 10,
            blocking_hold: None,
            async_hold: None,
            record_capacity_done: None,
        }
    }
}

#[derive(Clone)]
struct EffectsContext {
    expected_capacity: u8,
    blocking_hold: Option<Arc<BlockingHoldLatch>>,
    async_hold: Option<Arc<AsyncHoldLatch>>,
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
    RunAsyncHold,
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
    AsyncHold,
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

    fn context(service: &ServiceContext<EffectsArguments>) -> Result<EffectsContext, ServiceError> {
        Ok(EffectsContext {
            expected_capacity: service.arguments().capacity,
            blocking_hold: service.arguments().blocking_hold.clone(),
            async_hold: service.arguments().async_hold.clone(),
            record_capacity_done: service.arguments().record_capacity_done.clone(),
        })
    }

    fn build(
        service: &ServiceContext<EffectsArguments>,
        _context: &EffectsContext,
    ) -> Result<ServiceBuilder<Effects, Self::Context>, ServiceError> {
        let capacity = service.arguments().capacity;
        let blocking_io_applied = service
            .arguments()
            .blocking_hold
            .as_ref()
            .map(|latch| latch.io_applied.clone())
            .or_else(|| {
                service
                    .arguments()
                    .async_hold
                    .as_ref()
                    .map(|latch| latch.io_applied.clone())
            });
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
        .blocking_io(
            |io_context: &EffectsContext, _snapshot, request| match request {
                EffectsIoRequest::BlockingHold => {
                    let Some(latch) = &io_context.blocking_hold else {
                        return Err(IoError::new(
                            "BlockingHold requires a per-test BlockingHoldLatch",
                        ));
                    };
                    latch.started.send(()).map_err(|_| {
                        IoError::new("the test stopped waiting for blocking IO to start")
                    })?;
                    lock_unpoisoned(&latch.release).recv().map_err(|_| {
                        IoError::new("the test stopped before releasing blocking IO")
                    })?;
                    Ok(Some(EffectsIoResult::Succeeded))
                }
                EffectsIoRequest::Fail
                | EffectsIoRequest::Succeed
                | EffectsIoRequest::Panic
                | EffectsIoRequest::RecordCapacity
                | EffectsIoRequest::AsyncHold => Err(IoError::new("not a blocking IO request")),
            },
        )
        .io(
            |io_context: &EffectsContext, snapshot: &EffectsSnapshot, request| {
                let expected_capacity = io_context.expected_capacity;
                let snapshot_capacity = snapshot.capacity;
                let record_capacity_done = io_context.record_capacity_done.clone();
                let async_hold = io_context.async_hold.clone();
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
                        EffectsIoRequest::AsyncHold => {
                            let Some(latch) = &async_hold else {
                                return Err(IoError::new(
                                    "AsyncHold requires a per-test AsyncHoldLatch",
                                ));
                            };
                            latch.started.send(()).map_err(|_| {
                                IoError::new("the test stopped waiting for async IO to start")
                            })?;
                            let mut release = latch.release.lock().await;
                            release.recv().await.ok_or_else(|| {
                                IoError::new("the test stopped before releasing async IO")
                            })?;
                            Ok(Some(EffectsIoResult::Succeeded))
                        }
                    }
                }
            },
        )
        .command("RunIoChain", |_: EmptyRequest| {
            Ok(EffectsRequest::RunIoChain)
        })
        .command("RunIoPanic", |_: EmptyRequest| {
            Ok(EffectsRequest::RunIoPanic)
        })
        .command("ArmTimer", |_: EmptyRequest| {
            Ok(EffectsRequest::ArmTimer {
                after: Duration::from_secs(10),
            })
        })
        .command("ReArmTimer", |_: EmptyRequest| {
            Ok(EffectsRequest::ReArmTimer {
                after: Duration::from_secs(5),
            })
        })
        .command("CancelTimer", |_: EmptyRequest| {
            Ok(EffectsRequest::CancelTimer)
        })
        .command("ScheduleIoWithoutExecutor", |_: EmptyRequest| {
            Ok(EffectsRequest::ScheduleIoWithoutExecutor)
        })
        .command("RecordCapacity", |_: EmptyRequest| {
            Ok(EffectsRequest::RecordCapacity)
        })
        .command("RunBlockingHold", |_: EmptyRequest| {
            Ok(EffectsRequest::RunBlockingHold)
        })
        .command("RunAsyncHold", |_: EmptyRequest| {
            Ok(EffectsRequest::RunAsyncHold)
        })
        .query(
            "blocking_active",
            |_: EmptyRequest| Ok(EffectsQuery::BlockingActive),
            |active: bool| {
                Some(LevelQueryResponse {
                    level: u8::from(active),
                    max_level: 0,
                })
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
            Command::Request(EffectsRequest::RunAsyncHold) => {
                snapshot.blocking_io_running = true;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: vec![Effect::Io(EffectsIoRequest::AsyncHold)],
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

    fn context(_service: &ServiceContext<EffectsArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<EffectsArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Effects, Self::Context>, ServiceError> {
        let capacity = service.arguments().capacity;
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
            Ok(EffectsRequest::ScheduleIoWithoutExecutor)
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
        )
        .with_attachment(JobId::from_u128(1).to_string().into_bytes());
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
            .query::<EmptyRequest, LevelQueryResponse>("blocking_active", &EmptyRequest::default())
            .await
            .expect("the Query answers")
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
            .query::<EmptyRequest, LevelQueryResponse>("blocking_active", &EmptyRequest::default())
            .await
            .expect("the Query answers")
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

async fn start_effects_kernel_with_shutdown(
    arguments: EffectsArguments,
) -> (
    Arc<dyn blueos_comms::CommsBackend>,
    blueos_service::ShutdownHandle,
    tokio::task::JoinHandle<RunOutcome>,
) {
    let service = ServiceContext::new(arguments, blueos_service::testing::channel_session());
    let context = EffectsService::context(&service).expect("the effects context builds");
    let mut builder =
        EffectsService::build(&service, &context).expect("the effects service builds");
    let shutdown = builder.shutdown_handle();
    let backend: Arc<dyn blueos_comms::CommsBackend> =
        Arc::new(blueos_comms::channel::ChannelBackend::default());
    let kernel = Kernel::start(
        EffectsService::NAME,
        builder,
        context,
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    )
    .await
    .expect("the kernel starts");
    let run = tokio::spawn(async move { kernel.run().await });
    (backend, shutdown, run)
}

#[tokio::test(start_paused = true)]
async fn shutdown_waits_for_in_flight_io_before_returning() {
    let (started_sender, started_receiver) = mpsc::sync_channel(0);
    let (release_sender, release_receiver) = tokio::sync::mpsc::channel(1);
    let (io_applied_sender, io_applied_receiver) = mpsc::sync_channel(0);
    let latch = Arc::new(AsyncHoldLatch {
        started: started_sender,
        release: tokio::sync::Mutex::new(release_receiver),
        io_applied: io_applied_sender,
    });
    let (backend, shutdown, run) = start_effects_kernel_with_shutdown(EffectsArguments {
        async_hold: Some(Arc::clone(&latch)),
        ..EffectsArguments::default()
    })
    .await;
    let command = tokio::spawn(async move {
        let body = blueos_comms::QueryBody::new(
            EmptyRequest::default().encode().unwrap(),
            cdr_encoding(EmptyRequest::SCHEMA_NAME),
        )
        .with_attachment(JobId::from_u128(1).to_string().into_bytes());
        backend
            .get(
                &command_key(EffectsService::NAME, "RunAsyncHold"),
                Some(body),
                Duration::from_secs(10),
            )
            .await
            .unwrap();
    });
    tokio::task::spawn_blocking(move || started_receiver.recv())
        .await
        .expect("join")
        .expect("blocking IO starts");
    shutdown.trigger();
    advance(Duration::from_millis(1)).await;
    release_sender
        .send(())
        .await
        .expect("the test releases async IO");
    tokio::task::spawn_blocking(move || io_applied_receiver.recv())
        .await
        .expect("join")
        .expect("IO result reaches the Domain");
    command.await.expect("RunAsyncHold finishes");
    let outcome = timeout(Duration::from_secs(1), run)
        .await
        .expect("shutdown finishes without waiting the full drain budget")
        .expect("join");
    assert_eq!(outcome, RunOutcome::Stopped);
}

#[tokio::test(start_paused = true)]
async fn shutdown_abandons_in_flight_io_after_five_seconds() {
    let (started_sender, started_receiver) = mpsc::sync_channel(0);
    let (_release_sender, release_receiver) = tokio::sync::mpsc::channel(1);
    let (io_applied_sender, _io_applied_receiver) = mpsc::sync_channel(0);
    let latch = Arc::new(AsyncHoldLatch {
        started: started_sender,
        release: tokio::sync::Mutex::new(release_receiver),
        io_applied: io_applied_sender,
    });
    let (backend, shutdown, run) = start_effects_kernel_with_shutdown(EffectsArguments {
        async_hold: Some(Arc::clone(&latch)),
        ..EffectsArguments::default()
    })
    .await;
    let _command = tokio::spawn(async move {
        let body = blueos_comms::QueryBody::new(
            EmptyRequest::default().encode().unwrap(),
            cdr_encoding(EmptyRequest::SCHEMA_NAME),
        )
        .with_attachment(JobId::from_u128(1).to_string().into_bytes());
        backend
            .get(
                &command_key(EffectsService::NAME, "RunAsyncHold"),
                Some(body),
                Duration::from_secs(10),
            )
            .await
            .unwrap();
    });
    tokio::task::spawn_blocking(move || started_receiver.recv())
        .await
        .expect("join")
        .expect("blocking IO starts");
    let (finished_sender, mut finished_receiver) = tokio::sync::oneshot::channel();
    let run_task = tokio::spawn(async move {
        let outcome = run.await.expect("join");
        let _ = finished_sender.send(());
        outcome
    });
    shutdown.trigger();
    let ((), outcome) = tokio::join!(
        async {
            advance(Duration::from_millis(1)).await;
            advance(Duration::from_secs(4)).await;
            assert!(
                finished_receiver.try_recv().is_err(),
                "shutdown must not return before the five second IO drain budget elapses"
            );
            advance(Duration::from_secs(1)).await;
        },
        run_task,
    );
    assert_eq!(outcome.expect("join"), RunOutcome::Stopped);
}

#[tokio::test(start_paused = true)]
async fn shutdown_waits_for_in_flight_blocking_io_before_returning() {
    let (started_sender, started_receiver) = mpsc::sync_channel(0);
    let (release_sender, release_receiver) = mpsc::channel();
    let (io_applied_sender, io_applied_receiver) = mpsc::sync_channel(0);
    let latch = Arc::new(BlockingHoldLatch {
        started: started_sender,
        release: Mutex::new(release_receiver),
        io_applied: io_applied_sender,
    });
    let (backend, shutdown, run) = start_effects_kernel_with_shutdown(EffectsArguments {
        blocking_hold: Some(Arc::clone(&latch)),
        ..EffectsArguments::default()
    })
    .await;
    let command = tokio::spawn(async move {
        let body = blueos_comms::QueryBody::new(
            EmptyRequest::default().encode().unwrap(),
            cdr_encoding(EmptyRequest::SCHEMA_NAME),
        )
        .with_attachment(JobId::from_u128(1).to_string().into_bytes());
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
        .expect("join")
        .expect("blocking IO starts");
    shutdown.trigger();
    advance(Duration::from_millis(1)).await;
    release_sender
        .send(())
        .expect("the test releases blocking IO");
    tokio::task::spawn_blocking(move || io_applied_receiver.recv())
        .await
        .expect("join")
        .expect("IO result reaches the Domain");
    command.await.expect("RunBlockingHold finishes");
    let outcome = timeout(Duration::from_secs(1), run)
        .await
        .expect("shutdown finishes without waiting the full drain budget")
        .expect("join");
    assert_eq!(outcome, RunOutcome::Stopped);
}
