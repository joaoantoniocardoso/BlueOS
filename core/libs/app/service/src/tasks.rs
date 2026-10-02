//! Supervised Tasks: restart policies, shutdown joining and `status` health (D-27, D-04).

use core::{future::Future, panic::AssertUnwindSafe, pin::Pin, time::Duration};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use backon::{BackoffBuilder, ExponentialBuilder};
use bytes::Bytes;
use futures_util::{FutureExt, future::join_all};
use tokio::{sync::watch, task::JoinHandle};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::warn;

use blueos_api::Message;
use blueos_comms::{CommsBackend, Sample};
use blueos_domain::Domain;
use blueos_idl::{
    Error as IdlError,
    msg::blueos_msgs::{ServiceStatus, ServiceStatusStatus},
};

use crate::{
    clock::Clock,
    command_sender::{CommandSender, Session},
    inbox_recovery::{INBOX_LOOP_NAME, LoopPanicTracker, log_caught_panic},
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

/// Spawns every future on the shared [`TaskTracker`].
#[derive(Clone)]
pub(crate) struct TaskSpawner {
    tracker: Arc<TaskTracker>,
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
    handles: Mutex<Vec<(String, JoinHandle<()>)>>,
    status: StatusPublisher,
    loop_panics: LoopPanicTracker,
}

#[derive(Clone)]
struct StatusPublisher {
    service: &'static str,
    backend: Arc<dyn CommsBackend>,
    key: String,
    encoding: String,
    latest: watch::Sender<Option<Bytes>>,
    degraded_tasks: Arc<Mutex<BTreeSet<String>>>,
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
    pub(crate) fn new() -> Self {
        Self {
            tracker: Arc::new(TaskTracker::new()),
        }
    }

    pub(crate) fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tracker.spawn(future)
    }
}

impl TaskSupervisor {
    pub(crate) fn new(
        service: &'static str,
        backend: Arc<dyn CommsBackend>,
        status_key: String,
        status_encoding: String,
        status_latest: watch::Sender<Option<Bytes>>,
    ) -> Self {
        Self {
            spawner: TaskSpawner::new(),
            shutdown: CancellationToken::new(),
            handles: Mutex::new(Vec::new()),
            status: StatusPublisher {
                service,
                backend,
                key: status_key,
                encoding: status_encoding,
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
        for task in tasks {
            let name = task.name.clone();
            let policy = task.policy;
            let run = Arc::clone(&task.run);
            let shutdown = self.shutdown.child_token();
            let status = self.status.clone();
            let task_context = TaskContext {
                shutdown: shutdown.clone(),
                session: Arc::clone(&session),
                commands: commands.clone(),
                context: Arc::clone(&context),
                clock: Arc::clone(&clock),
            };
            let handle = self.spawner.spawn(supervise_task(
                name.clone(),
                policy,
                run,
                task_context,
                shutdown,
                Arc::clone(&clock),
                status,
            ));
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

impl StatusPublisher {
    async fn mark_running(&self, task: &str) {
        lock_unpoisoned(&self.degraded_tasks).remove(task);
        self.refresh_status().await;
    }

    async fn mark_degraded(&self, task: &str) {
        lock_unpoisoned(&self.degraded_tasks).insert(task.to_owned());
        self.refresh_status().await;
    }

    async fn refresh_status(&self) {
        let (status, detail) = {
            let degraded = lock_unpoisoned(&self.degraded_tasks);
            if degraded.is_empty() {
                (ServiceStatusStatus::Ready, String::new())
            } else {
                (
                    ServiceStatusStatus::Degraded,
                    degraded.iter().cloned().collect::<Vec<_>>().join(", "),
                )
            }
        };
        self.publish(status, detail).await;
    }

    async fn publish(&self, status: ServiceStatusStatus, detail: String) {
        let message = ServiceStatus { status, detail };
        let sent: Result<(), PublishError> = async {
            let payload = Bytes::from(message.encode()?);
            if self.latest.borrow().as_ref() == Some(&payload) {
                return Ok(());
            }
            let sample = Sample::new(
                self.key.as_str(),
                Bytes::clone(&payload),
                self.encoding.as_str(),
            );
            self.backend.publish(sample).await?;
            self.latest.send_replace(Some(payload));
            Ok(())
        }
        .await;
        if let Err(error) = sent {
            warn!(%error, key = %self.key, "Failed to publish service status");
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum PublishError {
    #[error("the Message does not encode: {0}")]
    Encode(#[from] IdlError),
    #[error(transparent)]
    Comms(#[from] blueos_comms::CommsError),
}

async fn supervise_task<D: Domain, Context: Send + Sync + 'static>(
    name: String,
    policy: RestartPolicy,
    run: TaskRun<D, Context>,
    task_context: TaskContext<D, Context>,
    shutdown: CancellationToken,
    clock: Arc<dyn Clock>,
    status: StatusPublisher,
) {
    let mut failures = 0u32;
    let mut delays = match policy {
        RestartPolicy::Never => None,
        RestartPolicy::OnFailure { backoff, .. } | RestartPolicy::Always { backoff } => {
            Some(backoff.delays())
        }
    };
    loop {
        if shutdown.is_cancelled() {
            break;
        }
        status.mark_running(&name).await;
        let context = TaskContext {
            shutdown: task_context.shutdown.clone(),
            session: Arc::clone(&task_context.session),
            commands: task_context.commands.clone(),
            context: Arc::clone(&task_context.context),
            clock: Arc::clone(&task_context.clock),
        };
        let run = Arc::clone(&run);
        let outcome = match AssertUnwindSafe(run(context)).catch_unwind().await {
            Ok(result) => result,
            Err(panic) => {
                log_caught_panic(status.service, Some(&name), panic);
                Err(TaskFailed)
            }
        };
        if shutdown.is_cancelled() {
            break;
        }
        let should_restart = match (outcome, policy) {
            (Ok(()), RestartPolicy::Never) => false,
            (Ok(()), RestartPolicy::OnFailure { .. }) => false,
            (Ok(()), RestartPolicy::Always { .. }) => true,
            (Err(TaskFailed), RestartPolicy::Never) => false,
            (Err(TaskFailed), RestartPolicy::OnFailure { max_attempts, .. }) => {
                failures = failures.saturating_add(1);
                if failures >= max_attempts {
                    status.mark_degraded(&name).await;
                    false
                } else {
                    true
                }
            }
            (Err(TaskFailed), RestartPolicy::Always { .. }) => true,
        };
        if !should_restart {
            break;
        }
        status.mark_degraded(&name).await;
        let Some(delay) = delays.as_mut().and_then(Iterator::next) else {
            break;
        };
        if wait_delay(&clock, &shutdown, delay).await {
            break;
        }
    }
}

async fn wait_delay(clock: &Arc<dyn Clock>, shutdown: &CancellationToken, delay: Duration) -> bool {
    let deadline = clock.now().monotonic + delay;
    while clock.now().monotonic < deadline {
        if shutdown.is_cancelled() {
            return true;
        }
        let remaining = deadline.saturating_sub(clock.now().monotonic);
        tokio::select! {
            biased;
            () = shutdown.cancelled() => return true,
            () = tokio::time::sleep(remaining) => {}
        }
    }
    false
}

/// Holds the liveliness token until shutdown; obeys the Task cancellation token.
pub(crate) async fn hold_liveliness_until_cancelled(
    liveliness: blueos_comms::LivelinessToken,
    shutdown: CancellationToken,
) {
    let _liveliness = liveliness;
    shutdown.cancelled().await;
}
