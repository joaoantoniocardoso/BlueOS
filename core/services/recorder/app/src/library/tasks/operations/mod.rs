//! Library operations Task: runs the repairs and snapshots the library wants, and stops a repair it no longer wants.

mod rewrite_work;

use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::task::{Id, JoinError, JoinSet};
use tracing::warn;

use blueos_domain::Command;
use blueos_recorder_domain::{RecorderDomain, RecorderObservedFact};
use blueos_recorder_library::{
    LibraryObservedFact, LibraryOperation, LibraryRepairOutcome, LibrarySnapshotOutcome,
    RepairFailure,
};
use blueos_service::{CommandSender, Projection, TaskContext, TaskFailed};

use crate::context::RecorderContext;

use rewrite_work::{finished_fact, repair, snapshot};

/// An operation whose rewrite runs on a blocking thread.
struct RunningOperation {
    /// Shared with the blocking task, which reports how it ended; kept here to report a rewrite that panicked.
    operation: Arc<LibraryOperation>,
    /// The blocking task that runs the rewrite.
    task_id: Id,
    /// Stops the rewrite, which then leaves the source untouched and removes its temporary file.
    cancel: Arc<AtomicBool>,
}

/// A blocking rewrite cannot be aborted, so dropping its handle, as an unwinding Task does, cancels it: the
/// restarted Task then never runs a second rewrite of the same recording beside it.
impl Drop for RunningOperation {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// Runs until shutdown, reconciling the rewrites it runs with the operations `operations` lists (D-27): it starts
/// each listed operation, cancels each running one the list dropped, and reports progress and each end as
/// Observed facts. At shutdown it cancels every rewrite and waits for them, so none outlives the Service.
pub(crate) async fn run_library_operations(
    task_context: TaskContext<RecorderDomain, RecorderContext>,
    operations: Projection<Vec<LibraryOperation>>,
) -> Result<(), TaskFailed> {
    let mut wanted = operations.subscribe();
    let mut running = Vec::new();
    let mut rewrites = JoinSet::new();
    loop {
        reconcile(
            &wanted.borrow_and_update(),
            &mut running,
            &mut rewrites,
            &task_context,
        );
        tokio::select! {
            () = task_context.shutdown.cancelled() => break,
            changed = wanted.changed() => {
                if changed.is_err() {
                    break;
                }
            }
            Some(joined) = rewrites.join_next_with_id() => {
                let Some(fact) = take_finished(joined, &mut running) else {
                    continue;
                };
                // The list still names the operation until the Domain applies its end; reconciling before that
                // would start it again.
                let ended = Command::ObservedFact(RecorderObservedFact::Library(fact));
                if task_context.commands.send_awaiting_ack(ended).await.is_err() {
                    break;
                }
            }
        }
    }
    drop(running);
    while rewrites.join_next().await.is_some() {}
    Ok(())
}

fn reconcile(
    wanted: &[LibraryOperation],
    running: &mut Vec<RunningOperation>,
    rewrites: &mut JoinSet<LibraryObservedFact>,
    task_context: &TaskContext<RecorderDomain, RecorderContext>,
) {
    for running_operation in running.iter() {
        let job_id = running_operation.operation.job_id();
        if !wanted.iter().any(|operation| operation.job_id() == job_id) {
            running_operation.cancel.store(true, Ordering::Relaxed);
        }
    }
    let to_start: Vec<Arc<LibraryOperation>> = wanted
        .iter()
        .filter(|operation| {
            !running
                .iter()
                .any(|running_operation| running_operation.operation.job_id() == operation.job_id())
        })
        .cloned()
        .map(Arc::new)
        .collect();
    for operation in to_start {
        let cancel = Arc::new(AtomicBool::new(false));
        let rewrite = rewrites.spawn_blocking({
            let operation = Arc::clone(&operation);
            let cancel = Arc::clone(&cancel);
            let recordings_folder = Arc::clone(&task_context.context.recordings_folder);
            let rewriter = Arc::clone(&task_context.context.rewriter);
            let commands = CommandSender::clone(&task_context.commands);
            move || {
                finished_fact(
                    &operation,
                    |path| repair(path, &recordings_folder, &rewriter, &commands, &cancel),
                    |path, output_path| {
                        snapshot(path, output_path, &recordings_folder, &rewriter, &cancel)
                    },
                )
            }
        });
        running.push(RunningOperation {
            operation,
            task_id: rewrite.id(),
            cancel,
        });
    }
}

/// Removes the rewrite that ended from `running` and returns how its operation ended.
fn take_finished(
    joined: Result<(Id, LibraryObservedFact), JoinError>,
    running: &mut Vec<RunningOperation>,
) -> Option<LibraryObservedFact> {
    let task_id = match &joined {
        Ok((task_id, _fact)) => *task_id,
        Err(error) => error.id(),
    };
    let index = running
        .iter()
        .position(|running_operation| running_operation.task_id == task_id)?;
    let finished = running.swap_remove(index);
    Some(match joined {
        Ok((_task_id, fact)) => fact,
        Err(error) => {
            warn!(%error, "Library operation rewrite panicked");
            finished_fact(
                &finished.operation,
                |_path| LibraryRepairOutcome::Failed(RepairFailure::Io),
                |_path, _output_path| LibrarySnapshotOutcome::Failed(RepairFailure::Io),
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use core::{panic::AssertUnwindSafe, time::Duration};
    use std::panic::catch_unwind;

    use blueos_jobs::JobId;
    use blueos_recorder_library::RepairProgress;
    use blueos_recorder_paths::RecordingRelativePath;

    use super::*;

    #[tokio::test]
    async fn a_task_that_unwinds_cancels_every_rewrite_it_runs() {
        let cancel = Arc::new(AtomicBool::new(false));
        let running_operation = RunningOperation {
            operation: Arc::new(LibraryOperation::Repair {
                path: RecordingRelativePath::parse("dive.mcap").expect("a valid path"),
                job_id: JobId::from_u128(1),
                progress: RepairProgress {
                    bytes_processed: 0,
                    total_bytes: 0,
                    started_monotonic: Duration::ZERO,
                },
            }),
            task_id: tokio::spawn(async {}).id(),
            cancel: Arc::clone(&cancel),
        };

        let unwound = catch_unwind(AssertUnwindSafe(move || {
            let _running = [running_operation];
            panic!("the operations Task panics");
        }));

        assert!(unwound.is_err());
        assert!(
            cancel.load(Ordering::Relaxed),
            "a rewrite must not outlive the Task that runs it"
        );
    }
}
