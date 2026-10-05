//! Supervised Task restart loop.

use core::{panic::AssertUnwindSafe, time::Duration};
use std::sync::Arc;

use futures_util::FutureExt;
use metrics::Counter;
use tokio_util::sync::CancellationToken;

use blueos_domain::Domain;

use crate::{clock::Clock, command_sender::CommandSender, inbox_recovery::log_caught_panic};

use super::{RestartPolicy, TaskContext, TaskFailed, TaskRun, status::StatusPublisher};

pub(super) struct SuperviseTaskWork<D: Domain, Context> {
    pub name: Arc<str>,
    pub policy: RestartPolicy,
    pub run: TaskRun<D, Context>,
    pub task_context: TaskContext<D, Context>,
    pub clock: Arc<dyn Clock>,
    pub status: Arc<StatusPublisher>,
    pub restarts: Counter,
}

struct RestartDecision {
    stop: bool,
    mark_degraded: bool,
}

pub(super) async fn supervise_task<D: Domain, Context: Send + Sync + 'static>(
    work: SuperviseTaskWork<D, Context>,
) {
    let SuperviseTaskWork {
        name,
        policy,
        run,
        task_context,
        clock,
        status,
        restarts,
    } = work;
    let mut failures = 0u32;
    let mut delays = match policy {
        RestartPolicy::Never => None,
        RestartPolicy::OnFailure { backoff, .. } | RestartPolicy::Always { backoff } => {
            Some(backoff.delays())
        }
    };
    let shared_commands = CommandSender::clone(&task_context.commands);
    let parent_shutdown = task_context.shutdown;
    loop {
        if parent_shutdown.is_cancelled() {
            break;
        }
        status.mark_running(name.as_ref()).await;
        let context = TaskContext {
            shutdown: parent_shutdown.child_token(),
            session: Arc::clone(&task_context.session),
            commands: CommandSender::clone(&shared_commands),
            context: Arc::clone(&task_context.context),
            clock: Arc::clone(&task_context.clock),
        };
        let run = Arc::clone(&run);
        let outcome = match AssertUnwindSafe(run(context)).catch_unwind().await {
            Ok(result) => result,
            Err(panic) => {
                log_caught_panic(status.service, Some(name.as_ref()), panic);
                Err(TaskFailed)
            }
        };
        if parent_shutdown.is_cancelled() {
            break;
        }
        let decision = restart_decision(outcome, policy, &mut failures);
        if decision.stop {
            if decision.mark_degraded {
                status.mark_degraded(name.as_ref()).await;
            }
            break;
        }
        status.mark_degraded(name.as_ref()).await;
        let Some(delay) = delays.as_mut().and_then(Iterator::next) else {
            break;
        };
        if wait_delay(&clock, &parent_shutdown, delay).await {
            break;
        }
        restarts.increment(1);
    }
}

fn restart_decision(
    outcome: Result<(), TaskFailed>,
    policy: RestartPolicy,
    failures: &mut u32,
) -> RestartDecision {
    let restart = match (outcome, policy) {
        (Ok(()), RestartPolicy::Never) => false,
        (Ok(()), RestartPolicy::OnFailure { .. }) => false,
        (Ok(()), RestartPolicy::Always { .. }) => true,
        (Err(TaskFailed), RestartPolicy::Never) => false,
        (Err(TaskFailed), RestartPolicy::OnFailure { max_attempts, .. }) => {
            *failures = failures.saturating_add(1);
            *failures < max_attempts
        }
        (Err(TaskFailed), RestartPolicy::Always { .. }) => true,
    };
    if restart {
        return RestartDecision {
            stop: false,
            mark_degraded: false,
        };
    }
    RestartDecision {
        stop: true,
        mark_degraded: matches!(
            (outcome, policy),
            (Err(TaskFailed), RestartPolicy::OnFailure { .. })
        ),
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
