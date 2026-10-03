//! Supervised Tasks: restart, `status` health and shutdown (layer L3).

use core::{
    convert::Infallible,
    future::pending,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use std::sync::{Arc, Mutex};

use clap::Args;
use tokio::time::timeout;
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    Layer, layer::SubscriberExt, registry::LookupSpan, util::SubscriberInitExt,
};

use blueos_api::status_state_key;
use blueos_comms::{CommsBackend, Subscriber as StateSubscriber};
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::{
    Message,
    msg::blueos_msgs::{ServiceStatus, ServiceStatusStatus},
};
use blueos_service::{
    Backoff, Clock, Kernel, RestartPolicy, RunOutcome, Service, ServiceBuilder, ServiceContext,
    ServiceError, TaskFailed,
    testing::{Harness, PausedClock, WALL_CLOCK_AT_START, lock_unpoisoned},
};

const RECV_TIMEOUT: Duration = Duration::from_secs(10);

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

    const NAME: &'static str = "tasks-harness";
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
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

struct WarningCapture(Arc<Mutex<Vec<String>>>);

impl<S: Subscriber> Layer<S> for WarningCapture
where
    S: for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if *event.metadata().level() > Level::WARN {
            return;
        }
        let mut message = String::new();
        let mut task = String::new();
        event.record(
            &mut |field: &tracing::field::Field, value: &dyn core::fmt::Debug| match field.name() {
                "message" => message = format!("{value:?}"),
                "task" => task = format!("{value:?}"),
                _ => {}
            },
        );
        let line = if task.is_empty() {
            message
        } else {
            format!("{message} {task}")
        };
        lock_unpoisoned(&self.0).push(line);
    }
}

async fn next_status(subscriber: &mut StateSubscriber) -> ServiceStatus {
    let sample = timeout(RECV_TIMEOUT, subscriber.recv())
        .await
        .expect("status update arrives before timeout")
        .expect("status stream stays open");
    ServiceStatus::decode(&sample.payload().to_bytes()).expect("status payload decodes")
}

async fn delay_between_first_two_attempts(policy: RestartPolicy) -> Duration {
    let (attempt_sender, mut attempt_receiver) = tokio::sync::mpsc::unbounded_channel();
    let builder =
        ServiceBuilder::<TasksDomain>::new(TasksSnapshot).task("flaky", policy, move |context| {
            let attempt_sender = attempt_sender.clone();
            async move {
                let _ = attempt_sender.send(context.clock.now().monotonic);
                Err(TaskFailed)
            }
        });
    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let kernel = Kernel::start(TasksService::NAME, builder, (), backend, clock)
        .await
        .expect("kernel starts");
    let run = tokio::spawn(kernel.run());
    let mut next_attempt = async || {
        timeout(RECV_TIMEOUT, attempt_receiver.recv())
            .await
            .expect("the Task runs before timeout")
            .expect("the Task keeps reporting attempts")
    };
    let first = next_attempt().await;
    let second = next_attempt().await;
    run.abort();
    second - first
}

#[tokio::test(start_paused = true)]
async fn always_restarts_after_the_backoff_the_task_declares() {
    let backoff = Backoff {
        minimum_delay: Duration::from_secs(1),
        maximum_delay: Duration::from_secs(1),
    };

    let delay = delay_between_first_two_attempts(RestartPolicy::Always { backoff }).await;

    assert!(delay >= Duration::from_secs(1), "restarted after {delay:?}");
}

#[test]
fn the_default_backoff_runs_from_100_milliseconds_to_30_seconds() {
    assert_eq!(
        Backoff::default(),
        Backoff {
            minimum_delay: Duration::from_millis(100),
            maximum_delay: Duration::from_secs(30),
        }
    );
}

#[tokio::test(start_paused = true)]
async fn on_failure_restarts_after_the_backoff_the_task_declares() {
    let backoff = Backoff {
        minimum_delay: Duration::from_secs(1),
        maximum_delay: Duration::from_secs(1),
    };

    let delay = delay_between_first_two_attempts(RestartPolicy::OnFailure {
        backoff,
        max_attempts: 3,
    })
    .await;

    assert!(delay >= Duration::from_secs(1), "restarted after {delay:?}");
}

#[tokio::test(start_paused = true)]
async fn always_failing_task_restarts_with_backoff_and_degrades_status() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_task = Arc::clone(&attempts);
    let mut builder = ServiceBuilder::<TasksDomain>::new(TasksSnapshot);
    builder = builder.task(
        "flaky",
        RestartPolicy::Always {
            backoff: Backoff::default(),
        },
        move |_context| {
            let attempt_counter = Arc::clone(&attempts_for_task);
            async move {
                attempt_counter.fetch_add(1, Ordering::SeqCst);
                Err(TaskFailed)
            }
        },
    );
    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let kernel = Kernel::start(
        TasksService::NAME,
        builder,
        (),
        Arc::clone(&backend),
        Arc::clone(&clock),
    )
    .await
    .expect("kernel starts");
    let mut status_subscriber = backend
        .subscribe(&status_state_key(TasksService::NAME))
        .await
        .expect("status key subscribes");
    let run = tokio::spawn(kernel.run());

    let mut degraded = next_status(&mut status_subscriber).await;
    if degraded.status != ServiceStatusStatus::Degraded {
        degraded = next_status(&mut status_subscriber).await;
    }
    assert_eq!(degraded.status, ServiceStatusStatus::Degraded);
    assert_eq!(degraded.detail, "flaky");

    tokio::time::advance(Duration::from_millis(100)).await;
    let ready_again = next_status(&mut status_subscriber).await;
    assert_eq!(ready_again.status, ServiceStatusStatus::Ready);

    assert!(attempts.load(Ordering::SeqCst) >= 2);

    run.abort();
}

#[tokio::test(start_paused = true)]
async fn status_names_remaining_task_while_the_other_restarts() {
    let beta_gate = Arc::new(tokio::sync::Notify::new());
    let beta_gate_for_task = Arc::clone(&beta_gate);
    let mut builder = ServiceBuilder::<TasksDomain>::new(TasksSnapshot);
    builder = builder
        .task(
            "alpha",
            RestartPolicy::Always {
                backoff: Backoff::default(),
            },
            |_context| async move { Err(TaskFailed) },
        )
        .task(
            "beta",
            RestartPolicy::Always {
                backoff: Backoff::default(),
            },
            move |_context| {
                let beta_notify = Arc::clone(&beta_gate_for_task);
                async move {
                    beta_notify.notified().await;
                    Err(TaskFailed)
                }
            },
        );
    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let kernel = Kernel::start(
        TasksService::NAME,
        builder,
        (),
        Arc::clone(&backend),
        Arc::clone(&clock),
    )
    .await
    .expect("kernel starts");
    let mut status_subscriber = backend
        .subscribe(&status_state_key(TasksService::NAME))
        .await
        .expect("status key subscribes");
    let run = tokio::spawn(kernel.run());

    let deadline = tokio::time::Instant::now() + RECV_TIMEOUT;
    loop {
        let status = next_status(&mut status_subscriber).await;
        if status.status == ServiceStatusStatus::Degraded && status.detail == "alpha" {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for status degraded with only alpha"
        );
    }

    beta_gate.notify_waiters();
    run.abort();
}

#[tokio::test(start_paused = true)]
async fn harness_build_can_declare_tasks() {
    let _harness = Harness::<TasksService>::start(TasksArguments {})
        .await
        .expect("harness starts");
    assert_eq!(PausedClock::start().now().wall, WALL_CLOCK_AT_START);
}

#[tokio::test(start_paused = true)]
async fn shutdown_leaves_no_tasks_running() {
    static RUNNING: AtomicUsize = AtomicUsize::new(0);
    let (started_sender, started_receiver) = tokio::sync::oneshot::channel();
    let started_sender = Arc::new(Mutex::new(Some(started_sender)));
    let started_for_task = Arc::clone(&started_sender);
    let mut builder = ServiceBuilder::<TasksDomain>::new(TasksSnapshot);
    builder = builder.task("worker", RestartPolicy::Never, move |context| {
        let started_for_task = Arc::clone(&started_for_task);
        async move {
            RUNNING.fetch_add(1, Ordering::SeqCst);
            if let Some(sender) = lock_unpoisoned(&started_for_task).take() {
                let _ = sender.send(());
            }
            context.shutdown.cancelled().await;
            RUNNING.fetch_sub(1, Ordering::SeqCst);
            Ok(())
        }
    });
    let shutdown = builder.shutdown_handle();
    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let kernel = Kernel::start(TasksService::NAME, builder, (), Arc::clone(&backend), clock)
        .await
        .expect("kernel starts");
    let run = tokio::spawn(async move { kernel.run().await });
    timeout(RECV_TIMEOUT, started_receiver)
        .await
        .expect("worker starts before timeout")
        .expect("worker signals start");
    assert_eq!(RUNNING.load(Ordering::SeqCst), 1);
    shutdown.trigger();
    tokio::time::advance(Duration::from_secs(5)).await;
    assert_eq!(run.await.expect("kernel run finishes"), RunOutcome::Stopped);
    assert_eq!(RUNNING.load(Ordering::SeqCst), 0);
}

#[tokio::test(start_paused = true)]
async fn straggler_is_aborted_and_named_in_warning() {
    let warnings = Arc::new(Mutex::new(Vec::new()));
    let _subscriber = tracing_subscriber::registry()
        .with(WarningCapture(Arc::clone(&warnings)))
        .set_default();

    let mut builder = ServiceBuilder::<TasksDomain>::new(TasksSnapshot);
    builder = builder.task("straggler", RestartPolicy::Never, |_| async {
        pending::<()>().await;
        Ok(())
    });
    let shutdown = builder.shutdown_handle();
    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let kernel = Kernel::start(TasksService::NAME, builder, (), Arc::clone(&backend), clock)
        .await
        .expect("kernel starts");
    let run = tokio::spawn(async move { kernel.run().await });
    shutdown.trigger();
    tokio::time::advance(Duration::from_secs(5)).await;
    assert_eq!(run.await.expect("kernel run finishes"), RunOutcome::Stopped);
    let captured = lock_unpoisoned(&warnings).clone();
    assert!(
        captured.iter().any(|message| message.contains("straggler")),
        "expected a warning naming the straggler, got {captured:?}"
    );
}
