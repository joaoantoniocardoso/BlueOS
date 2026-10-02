//! Inbox loop and Task panic recovery (layer L3, D-29).

use core::{
    convert::Infallible,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use std::sync::{Arc, Mutex};

use clap::Args;
use tokio::time::timeout;

use blueos_api::{Message, cdr_encoding, command_key, status_state_key};
use blueos_comms::{CommsBackend, QueryBody};
use blueos_domain::{Command, Decision, Domain, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::{PumpState, SetLevelRequest},
    blueos_msgs::{ServiceStatus, ServiceStatusStatus},
};
use blueos_service::{
    Backoff, Clock, Kernel, RestartPolicy, RunOutcome, Service, ServiceBuilder, ServiceContext,
    ServiceError, TaskFailed,
    testing::{Harness, PausedClock},
};

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
    capacity: u8,
    level_set_at: Duration,
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

    const NAME: &'static str = "recovery-tank";
    const VERSION: &'static str = "1.0.0";

    fn build(
        context: &ServiceContext<TankArguments>,
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        let capacity = context.arguments().capacity;
        Ok(ServiceBuilder::new(TankSnapshot {
            level: 0,
            capacity,
            level_set_at: Duration::ZERO,
        })
        .command("SetLevel", |request: SetLevelRequest| {
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

struct TasksService;

#[derive(Args, Clone)]
struct TasksArguments {}

struct TasksDomain;

#[derive(Clone, Default)]
struct TasksSnapshot;

impl Service for TasksService {
    type Domain = TasksDomain;
    type Context = ();
    type Arguments = TasksArguments;

    const NAME: &'static str = "recovery-tasks";
    const VERSION: &'static str = "1.0.0";

    fn build(
        _context: &ServiceContext<TasksArguments>,
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

async fn next_status(subscriber: &mut blueos_comms::Subscriber) -> ServiceStatus {
    let sample = timeout(RECV_TIMEOUT, subscriber.recv())
        .await
        .expect("status update arrives before timeout")
        .expect("status stream stays open");
    ServiceStatus::decode(&sample.payload().to_bytes()).expect("status payload decodes")
}

#[tokio::test(start_paused = true)]
async fn inbox_loop_panic_marks_status_degraded_and_keeps_running() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .expect("harness starts");
    let backend = harness.backend();
    let mut status_subscriber = backend
        .subscribe(&status_state_key(TankService::NAME))
        .await
        .expect("status key subscribes");

    harness
        .send("SetLevel", &SetLevelRequest { level: 1 })
        .await;

    let ack = harness
        .send(
            "SetLevel",
            &SetLevelRequest {
                level: LEVEL_THAT_PANICS_IN_HANDLE,
            },
        )
        .await;
    assert!(!ack.accepted);

    let degraded = next_status(&mut status_subscriber).await;
    assert_eq!(degraded.status, ServiceStatusStatus::Degraded);
    assert_eq!(degraded.detail, "inbox");

    harness
        .send("SetLevel", &SetLevelRequest { level: 2 })
        .await;
    let ready = next_status(&mut status_subscriber).await;
    assert_eq!(ready.status, ServiceStatusStatus::Ready);
    assert_eq!(harness.state::<PumpState>("tank").await.level, 2);
}

#[tokio::test(start_paused = true)]
async fn three_inbox_loop_panics_within_one_minute_exit_non_zero() {
    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let builder = TankService::build(&ServiceContext::new(
        TankArguments { capacity: 100 },
        blueos_service::testing::channel_session(),
    ))
    .expect("build");
    let kernel = Kernel::start(
        TankService::NAME,
        builder,
        Arc::clone(&backend),
        Arc::clone(&clock),
    )
    .await
    .expect("kernel starts");
    let run = tokio::spawn(kernel.run());

    let client = tokio::spawn({
        let backend = Arc::clone(&backend);
        async move {
            for _ in 0..3 {
                let body = QueryBody::new(
                    SetLevelRequest {
                        level: LEVEL_THAT_PANICS_IN_HANDLE,
                    }
                    .encode()
                    .expect("encode"),
                    cdr_encoding(SetLevelRequest::SCHEMA_NAME),
                );
                backend
                    .get(
                        &command_key(TankService::NAME, "SetLevel"),
                        Some(body),
                        RECV_TIMEOUT,
                    )
                    .await
                    .expect("command key is valid");
            }
        }
    });

    assert_eq!(
        timeout(RECV_TIMEOUT, run)
            .await
            .expect("kernel finishes")
            .expect("join"),
        RunOutcome::RepeatedInboxPanics
    );
    client.abort();
}

#[tokio::test(start_paused = true)]
async fn poisoned_lock_does_not_stop_another_task() {
    let shared = Arc::new(Mutex::new(0u32));
    let survivor_finished = Arc::new(tokio::sync::Notify::new());
    let shared_for_survivor = Arc::clone(&shared);
    let successes_for_survivor = Arc::new(AtomicUsize::new(0));
    let survivor_finished_for_task = Arc::clone(&survivor_finished);
    let shared_for_poisoner = Arc::clone(&shared);
    let poisoner = std::thread::spawn(move || {
        let _guard = shared_for_poisoner.lock().unwrap();
        panic!("poisoner panicked while holding the lock");
    });
    assert!(poisoner.join().is_err());
    assert!(shared.is_poisoned());

    let mut builder = ServiceBuilder::<TasksDomain>::new(TasksSnapshot);
    builder = builder.task(
        "survivor",
        RestartPolicy::Always {
            backoff: Backoff::default(),
        },
        move |_task_context| {
            let shared_for_survivor = Arc::clone(&shared_for_survivor);
            let successes_for_survivor = Arc::clone(&successes_for_survivor);
            let survivor_finished_for_task = Arc::clone(&survivor_finished_for_task);
            async move {
                let mut guard = shared_for_survivor
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *guard += 1;
                if successes_for_survivor.fetch_add(1, Ordering::SeqCst) + 1 >= 2 {
                    survivor_finished_for_task.notify_one();
                }
                Err(TaskFailed)
            }
        },
    );

    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let kernel = Kernel::start(
        TasksService::NAME,
        builder,
        Arc::clone(&backend),
        Arc::clone(&clock),
    )
    .await
    .expect("kernel starts");
    let run = tokio::spawn(kernel.run());

    timeout(RECV_TIMEOUT, survivor_finished.notified())
        .await
        .expect("survivor restarts before timeout");

    run.abort();
}
