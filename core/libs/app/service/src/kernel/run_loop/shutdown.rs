//! Shutdown drain: empty the Inbox while IO chains finish.

use core::time::Duration;

use tracing::warn;

use blueos_domain::Domain;

use crate::{run_outcome::RunOutcome, shutdown::SHUTDOWN_IO_DRAIN_TIMEOUT};

use super::super::types::Kernel;

pub(super) enum ShutdownStep {
    KeepDraining,
    Finished,
    Stop(RunOutcome),
}

pub(super) async fn drain_one_turn<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    shutdown_monotonic_deadline: &mut Option<Duration>,
    repeated_inbox_panics: &mut Option<RunOutcome>,
) -> ShutdownStep {
    let remaining = shutdown_monotonic_deadline
        .map(|deadline| deadline.saturating_sub(kernel.clock.now().monotonic))
        .unwrap_or(SHUTDOWN_IO_DRAIN_TIMEOUT);
    while let Ok(Some(delivery)) = tokio::time::timeout(Duration::ZERO, kernel.inbox.recv()).await {
        if let Some(outcome) = kernel.dispatch(delivery).await {
            *repeated_inbox_panics = Some(outcome);
            return ShutdownStep::Stop(outcome);
        }
    }
    if kernel.io_inflight.count() == 0 {
        let task_budget = shutdown_monotonic_deadline
            .map(|deadline| deadline.saturating_sub(kernel.clock.now().monotonic))
            .unwrap_or(SHUTDOWN_IO_DRAIN_TIMEOUT);
        kernel
            .tasks
            .join_with_budget(task_budget, &kernel.clock)
            .await;
        return ShutdownStep::Finished;
    }
    if remaining == Duration::ZERO {
        warn!(
            timeout = ?SHUTDOWN_IO_DRAIN_TIMEOUT,
            "shutdown io drain timed out"
        );
        kernel
            .tasks
            .join_with_budget(Duration::ZERO, &kernel.clock)
            .await;
        return ShutdownStep::Finished;
    }
    tokio::select! {
        biased;
        () = tokio::time::sleep(remaining) => {
            warn!(
                timeout = ?SHUTDOWN_IO_DRAIN_TIMEOUT,
                "shutdown io drain timed out"
            );
            kernel.tasks.join_with_budget(Duration::ZERO, &kernel.clock).await;
            ShutdownStep::Finished
        }
        delivery = kernel.inbox.recv() => {
            if let Some(delivery) = delivery
                && let Some(outcome) = kernel.dispatch(delivery).await {
                    *repeated_inbox_panics = Some(outcome);
                    return ShutdownStep::Stop(outcome);
                }
            ShutdownStep::KeepDraining
        }
    }
}
