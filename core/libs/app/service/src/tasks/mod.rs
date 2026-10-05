//! Supervised Tasks: restart policies, shutdown joining and `status` health (D-27, D-04).

mod status;
mod supervise;

use status::StatusPublisher;
use supervise::supervise_task;

pub(crate) use supervise::hold_liveliness_until_cancelled;

use core::{future::Future, pin::Pin, time::Duration};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use backon::{BackoffBuilder, ExponentialBuilder};
use bytes::Bytes;
use futures_util::future::join_all;
use tokio::{sync::watch, task::JoinHandle};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::warn;

use blueos_api::{Message, cdr_encoding};
use blueos_comms::CommsBackend;
use blueos_domain::Domain;
use blueos_idl::msg::blueos_msgs::ServiceStatus;

use crate::{
    clock::Clock,
    command_sender::{CommandSender, Session},
    inbox_recovery::{INBOX_LOOP_NAME, LoopPanicTracker},
    metrics_registry::MetricsRegistry,
    sync::lock_unpoisoned,
};

/// Minimum delay between Task restart attempts (spec D-27).
const BACKOFF_MIN: Duration = Duration::from_millis(100);
/// Maximum delay between Task restart attempts (spec D-27).
const BACKOFF_MAX: Duration = Duration::from_secs(30);

/// When a supervised Task body fails and should be restarted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskFailed;

/// How the Kernel restarts a Task when it stops or fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestartPolicy {
    /// Run the Task once; no restart.
    Never,
    /// Restart only after failure, up to `max_attempts` failures.
    OnFailure {
        /// How long the supervisor waits before each restart.
        backoff: Backoff,
        /// How many times the Task may fail before the supervisor stops restarting it.
        max_attempts: u32,
    },
    /// Restart whenever the Task stops.
    Always {
        /// How long the supervisor waits before each restart.
        backoff: Backoff,
    },
}

/// The delay before each Task restart: it doubles from `minimum_delay` up to `maximum_delay`, plus up to as much
/// again of random jitter, so Tasks that fail together do not restart together.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Backoff {
    /// The delay before the first restart, before jitter.
    pub minimum_delay: Duration,
    /// The longest delay between restarts, before jitter.
    pub maximum_delay: Duration,
}

/// Context passed to every Task body.
pub struct TaskContext<D: Domain, Context> {
    /// Becomes cancelled when the service begins shutdown.
    pub shutdown: CancellationToken,
    /// The service's Session to the backbone.
    pub session: Session,
    /// Puts Commands into this service's Inbox.
    pub commands: CommandSender<D>,
    /// The service Context from `build`.
    pub context: Arc<Context>,
    /// Injected clock for timestamps and timers (D-29).
    pub clock: Arc<dyn Clock>,
}

/// Spawns every future on the shared [`TaskTracker`], recording its metrics in the Service's registry.
#[derive(Clone)]
pub(crate) struct TaskSpawner {
    tracker: Arc<TaskTracker>,
    metrics: MetricsRegistry,
}

/// One Task declared in `build`.
pub(crate) struct TaskSpec<D: Domain, Context> {
    pub(crate) name: String,
    pub(crate) policy: RestartPolicy,
    pub(crate) run: TaskRun<D, Context>,
}

/// Runs one Task attempt.
pub(crate) type TaskRun<D, Context> = Arc<
    dyn Fn(TaskContext<D, Context>) -> Pin<Box<dyn Future<Output = Result<(), TaskFailed>> + Send>>
        + Send
        + Sync,
>;

/// Owns supervised Tasks, publishes `status`, and joins or aborts them on shutdown.
pub(crate) struct TaskSupervisor {
    spawner: TaskSpawner,
    shutdown: CancellationToken,
    handles: Mutex<Vec<(Arc<str>, JoinHandle<()>)>>,
    status: StatusPublisher,
    loop_panics: LoopPanicTracker,
}

impl Default for Backoff {
    /// Exponential backoff from 100 ms to 30 s with jitter (D-27).
    fn default() -> Self {
        Self {
            minimum_delay: BACKOFF_MIN,
            maximum_delay: BACKOFF_MAX,
        }
    }
}

impl Backoff {
    fn delays(self) -> impl Iterator<Item = Duration> + Send {
        ExponentialBuilder::new()
            .with_min_delay(self.minimum_delay)
            .with_max_delay(self.maximum_delay)
            .with_jitter()
            .without_max_times()
            .build()
    }
}

impl TaskSpawner {
    pub(crate) fn new(metrics: MetricsRegistry) -> Self {
        Self {
            tracker: Arc::new(TaskTracker::new()),
            metrics,
        }
    }

    pub(crate) fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tracker.spawn(self.metrics.scope(future))
    }
}

impl TaskSupervisor {
    pub(crate) fn new(
        service: &'static str,
        backend: Arc<dyn CommsBackend>,
        status_key: String,
        status_latest: watch::Sender<Option<Bytes>>,
        metrics: MetricsRegistry,
    ) -> Self {
        Self {
            spawner: TaskSpawner::new(metrics),
            shutdown: CancellationToken::new(),
            handles: Mutex::new(Vec::new()),
            status: StatusPublisher {
                service,
                backend,
                key: status_key,
                encoding: cdr_encoding(ServiceStatus::SCHEMA_NAME),
                latest: status_latest,
                degraded_tasks: Arc::new(Mutex::new(BTreeSet::new())),
            },
            loop_panics: LoopPanicTracker::new(),
        }
    }

    pub(crate) async fn mark_inbox_loop_degraded(&self) {
        self.status.mark_degraded(INBOX_LOOP_NAME).await;
    }

    pub(crate) async fn mark_inbox_loop_healthy(&self) {
        self.status.mark_running(INBOX_LOOP_NAME).await;
    }

    /// Records an Inbox loop recovery. Returns true when the Kernel should exit (D-29).
    pub(crate) async fn record_inbox_loop_panic(&self, monotonic_now: Duration) -> bool {
        self.mark_inbox_loop_degraded().await;
        self.loop_panics.record_recovery(monotonic_now)
    }

    pub(crate) fn spawner(&self) -> TaskSpawner {
        self.spawner.clone()
    }

    pub(crate) fn shutdown_token(&self) -> CancellationToken {
        self.shutdown.clone()
    }

    pub(crate) fn start<D: Domain, Context: Send + Sync + 'static>(
        &self,
        tasks: Vec<TaskSpec<D, Context>>,
        session: Session,
        commands: CommandSender<D>,
        context: Arc<Context>,
        clock: Arc<dyn Clock>,
    ) {
        let status = Arc::new(self.status.clone());
        let shared_commands = CommandSender::clone(&commands);
        for task in tasks {
            let name: Arc<str> = Arc::from(task.name);
            let policy = task.policy;
            let run = task.run;
            let restart_label = name.to_string();
            let restarts = metrics::with_local_recorder(
                &self.spawner.metrics,
                move || metrics::counter!("task_restarts", "task" => restart_label),
            );
            let task_context = TaskContext {
                shutdown: self.shutdown.child_token(),
                session: Arc::clone(&session),
                commands: CommandSender::clone(&shared_commands),
                context: Arc::clone(&context),
                clock: Arc::clone(&clock),
            };
            let handle = self
                .spawner
                .spawn(supervise_task(supervise::SuperviseTaskWork {
                    name: Arc::clone(&name),
                    policy,
                    run,
                    task_context,
                    clock: Arc::clone(&clock),
                    status: Arc::clone(&status),
                    restarts,
                }));
            lock_unpoisoned(&self.handles).push((name, handle));
        }
    }

    pub(crate) fn cancel(&self) {
        self.shutdown.cancel();
    }

    pub(crate) async fn join_with_budget(&self, budget: Duration, clock: &Arc<dyn Clock>) {
        let mut taken = core::mem::take(&mut *lock_unpoisoned(&self.handles));
        if budget > Duration::ZERO {
            let remaining = {
                let deadline = clock.now().monotonic + budget;
                deadline.saturating_sub(clock.now().monotonic)
            };
            let join = join_all(
                taken
                    .iter_mut()
                    .filter(|(_, handle)| !handle.is_finished())
                    .map(|(_, handle)| handle),
            );
            tokio::select! {
                () = tokio::time::sleep(remaining) => {}
                _ = join => {}
            }
        }
        for (name, handle) in taken {
            if handle.is_finished() {
                continue;
            }
            handle.abort();
            warn!(task = %name, "Shutdown aborted a Task that did not finish in time");
        }
    }
}
