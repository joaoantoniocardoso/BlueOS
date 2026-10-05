//! Post-loop cleanup: drain inbox, durable flush, logging shutdown.

use core::time::Duration;

use blueos_domain::Domain;

use crate::run_outcome::RunOutcome;

use super::super::types::Kernel;

pub(super) async fn finalize<D: Domain, Context: Send + Sync + 'static>(
    mut kernel: Kernel<D, Context>,
    repeated_inbox_panics: Option<RunOutcome>,
) -> RunOutcome {
    kernel.inbox_sender.take();
    kernel.endpoints.abort_all();
    while let Ok(Some(delivery)) = tokio::time::timeout(Duration::ZERO, kernel.inbox.recv()).await {
        if let Some(outcome) = kernel.dispatch(delivery).await {
            return outcome;
        }
    }
    if let Some(durable) = &mut kernel.durable {
        durable.persister.flush_and_shutdown().await;
    }
    if let Some(runtime) = kernel.log_publisher.take() {
        runtime.shutdown_and_wait().await;
    }
    repeated_inbox_panics.unwrap_or(RunOutcome::Stopped)
}
