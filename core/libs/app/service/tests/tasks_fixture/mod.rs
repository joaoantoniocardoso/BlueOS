#![expect(
    unreachable_pub,
    reason = "fixture items are re-exported to integration test roots"
)]
#![expect(
    dead_code,
    reason = "fixture items are shared across sibling integration test binaries"
)]

use core::{convert::Infallible, time::Duration};
use std::sync::{Arc, Mutex};

use clap::Args;
use tokio::time::timeout;
use tracing::{Level, Subscriber};
use tracing_subscriber::{Layer, registry::LookupSpan};

use blueos_api::status_state_key;
use blueos_comms::{CommsBackend, Subscriber as StateSubscriber};
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::{Message, msg::blueos_msgs::ServiceStatus};
use blueos_service::{
    Clock, Kernel, RestartPolicy, RunOutcome, Service, ServiceBuilder, ServiceContext,
    ServiceError, TaskFailed,
    testing::{PausedClock, lock_unpoisoned},
};

pub const RECV_TIMEOUT: Duration = Duration::from_secs(10);

pub struct TasksService;

#[derive(Args, Clone)]
pub struct TasksArguments {}

pub struct TasksDomain;

#[derive(Clone, Default)]
pub struct TasksSnapshot;

pub struct WarningCapture(pub Arc<Mutex<Vec<String>>>);

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

pub async fn start_tasks_kernel(
    builder: ServiceBuilder<TasksDomain>,
) -> (
    Arc<dyn CommsBackend>,
    tokio::task::JoinHandle<RunOutcome>,
    StateSubscriber,
) {
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
    let status_subscriber = backend
        .subscribe(&status_state_key(TasksService::NAME))
        .await
        .expect("status key subscribes");
    let run = tokio::spawn(kernel.run());
    (backend, run, status_subscriber)
}

pub async fn next_status(subscriber: &mut StateSubscriber) -> ServiceStatus {
    let sample = timeout(RECV_TIMEOUT, subscriber.recv())
        .await
        .expect("status update arrives before timeout")
        .expect("status stream stays open");
    ServiceStatus::decode(&sample.payload().to_bytes()).expect("status payload decodes")
}

pub async fn delay_between_first_two_attempts(policy: RestartPolicy) -> Duration {
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
