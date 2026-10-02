//! Forwards library repair progress Observed facts into the Inbox.

use std::sync::Arc;

use blueos_domain::Command;
use blueos_recorder_domain::RecorderDomain;
use blueos_service::{TaskContext, TaskFailed};

use crate::context::RecorderContext;

/// Reads repair progress from library IO and queues Observed facts on the Inbox.
pub(crate) async fn run_library_observed_bridge(
    task_context: TaskContext<RecorderDomain, RecorderContext>,
) -> Result<(), TaskFailed> {
    let receiver_mutex = Arc::clone(&task_context.context.library_observed_receiver);
    let mut receiver = receiver_mutex.lock().await;
    while let Some(fact) = receiver.recv().await {
        if task_context
            .commands
            .send(Command::ObservedFact(fact))
            .await
            .is_err()
        {
            break;
        }
    }
    Ok(())
}
