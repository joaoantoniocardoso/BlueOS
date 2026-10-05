//! Library operations Task: runs the repairs and snapshots the library wants, and stops a repair it no longer wants.

use core::sync::atomic::{AtomicBool, Ordering};
use std::{fs, io, path::Path, sync::Arc};

use tokio::task::{Id, JoinError, JoinSet};
use tracing::{debug, warn};

use blueos_domain::Command;
use blueos_recorder_domain::{RecorderDomain, RecorderObservedFact};
use blueos_recorder_library::{
    LibraryObservedFact, LibraryOperation, LibraryRepairOutcome, LibraryRepairProgress,
    LibrarySnapshotOutcome, RepairFailure,
};
use blueos_recorder_mcap::RewriteError;
use blueos_recorder_paths::RecordingRelativePath;
use blueos_recorder_storage::RecordingsFolder;
use blueos_service::{CommandSender, Projection, TaskContext, TaskFailed};

use crate::context::{RecorderContext, Rewriter};

/// An operation whose rewrite runs on a blocking thread.
struct RunningOperation {
    operation: LibraryOperation,
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
    for operation in wanted {
        if running
            .iter()
            .any(|running_operation| running_operation.operation.job_id() == operation.job_id())
        {
            continue;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let rewrite = rewrites.spawn_blocking({
            let operation = operation.clone();
            let cancel = Arc::clone(&cancel);
            let recordings_folder = Arc::clone(&task_context.context.recordings_folder);
            let rewriter = Arc::clone(&task_context.context.rewriter);
            let commands = task_context.commands.clone();
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
            operation: operation.clone(),
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

/// The fact that `operation` ended, with the outcome `repair_outcome` or `snapshot_outcome` gives it.
fn finished_fact(
    operation: &LibraryOperation,
    repair_outcome: impl FnOnce(&RecordingRelativePath) -> LibraryRepairOutcome,
    snapshot_outcome: impl FnOnce(&RecordingRelativePath, &str) -> LibrarySnapshotOutcome,
) -> LibraryObservedFact {
    match operation {
        LibraryOperation::Repair { path, .. } => LibraryObservedFact::RepairFinished {
            path: path.clone(),
            outcome: repair_outcome(path),
        },
        LibraryOperation::Snapshot {
            path, output_path, ..
        } => LibraryObservedFact::SnapshotFinished {
            path: path.clone(),
            output_path: output_path.clone(),
            outcome: snapshot_outcome(path, output_path),
        },
    }
}

/// Rewrites `path` into its `.recover` file and replaces the recording with it, unless cancelled first.
fn repair(
    path: &RecordingRelativePath,
    recordings_folder: &RecordingsFolder,
    rewriter: &Rewriter,
    commands: &CommandSender<RecorderDomain>,
    cancel: &AtomicBool,
) -> LibraryRepairOutcome {
    let relative = path.as_str();
    let source = match recordings_folder.resolve(relative) {
        Ok(source) => source,
        Err(error) => {
            warn!(%error, path = %relative, "Failed to resolve a recording to repair");
            return LibraryRepairOutcome::Failed(RepairFailure::Io);
        }
    };
    let temporary = recordings_folder.recover_temporary_path(relative);
    let rewritten = rewriter(
        &source,
        &temporary,
        &mut |bytes_processed, total_bytes| {
            let progress = LibraryObservedFact::RepairProgress(LibraryRepairProgress {
                path: path.clone(),
                bytes_processed,
                total_bytes,
            });
            // A progress fact dropped on a full Inbox is replaced by the next one.
            if let Err(error) = commands.try_send(Command::ObservedFact(
                RecorderObservedFact::Library(progress),
            )) {
                debug!(%error, path = %relative, "Dropped repair progress");
            }
        },
        cancel,
    );
    let outcome = match rewritten {
        Ok(_summary) if cancel.load(Ordering::Relaxed) => LibraryRepairOutcome::Cancelled,
        Ok(_summary) => {
            match recordings_folder.replace_recording_from_temporary(&temporary, relative) {
                Ok(()) => LibraryRepairOutcome::Succeeded,
                Err(error) => {
                    warn!(%error, path = %relative, "Failed to replace a recording with its repair");
                    LibraryRepairOutcome::Failed(RepairFailure::Replace)
                }
            }
        }
        Err(RewriteError::Cancelled) => LibraryRepairOutcome::Cancelled,
        Err(RewriteError::NotMcap) => LibraryRepairOutcome::Failed(RepairFailure::NotMcap),
        Err(error @ RewriteError::Mcap(_)) => {
            warn!(%error, path = %relative, "Failed to rewrite a recording to repair");
            LibraryRepairOutcome::Failed(RepairFailure::Rewrite)
        }
        Err(RewriteError::Io(_)) => LibraryRepairOutcome::Failed(RepairFailure::Io),
    };
    if outcome != LibraryRepairOutcome::Succeeded {
        remove_temporary(&temporary);
    }
    outcome
}

/// Rewrites `path` into an indexed copy at `output_path`.
fn snapshot(
    path: &RecordingRelativePath,
    output_path: &str,
    recordings_folder: &RecordingsFolder,
    rewriter: &Rewriter,
    cancel: &AtomicBool,
) -> LibrarySnapshotOutcome {
    let source = match recordings_folder.resolve(path.as_str()) {
        Ok(source) => source,
        Err(error) => {
            warn!(%error, path = path.as_str(), "Failed to resolve a recording to snapshot");
            return LibrarySnapshotOutcome::Failed(RepairFailure::Io);
        }
    };
    let temporary = recordings_folder.snapshot_temporary_path(output_path);
    let outcome = match rewriter(&source, &temporary, &mut |_read, _total| {}, cancel) {
        Ok(_summary) => match recordings_folder.finalize_snapshot(&temporary, output_path) {
            Ok(()) => LibrarySnapshotOutcome::Succeeded,
            Err(error) => {
                warn!(%error, path = %output_path, "Failed to finish a snapshot");
                LibrarySnapshotOutcome::Failed(RepairFailure::Replace)
            }
        },
        Err(RewriteError::Cancelled) => LibrarySnapshotOutcome::Failed(RepairFailure::Rewrite),
        Err(RewriteError::NotMcap) => LibrarySnapshotOutcome::Failed(RepairFailure::NotMcap),
        Err(error @ RewriteError::Mcap(_)) => {
            warn!(%error, path = path.as_str(), "Failed to rewrite a recording to snapshot");
            LibrarySnapshotOutcome::Failed(RepairFailure::Rewrite)
        }
        Err(RewriteError::Io(_)) => LibrarySnapshotOutcome::Failed(RepairFailure::Io),
    };
    if outcome != LibrarySnapshotOutcome::Succeeded {
        remove_temporary(&temporary);
    }
    outcome
}

fn remove_temporary(temporary: &Path) {
    match fs::remove_file(temporary) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            warn!(%error, path = %temporary.display(), "Failed to remove a temporary rewrite file");
        }
    }
}

#[cfg(test)]
mod tests {
    use core::{panic::AssertUnwindSafe, time::Duration};
    use std::panic::catch_unwind;

    use blueos_jobs::JobId;
    use blueos_recorder_library::RepairProgress;

    use super::*;

    #[tokio::test]
    async fn a_task_that_unwinds_cancels_every_rewrite_it_runs() {
        let cancel = Arc::new(AtomicBool::new(false));
        let running_operation = RunningOperation {
            operation: LibraryOperation::Repair {
                path: RecordingRelativePath::parse("dive.mcap").expect("a valid path"),
                job_id: JobId::from_u128(1),
                progress: RepairProgress {
                    bytes_processed: 0,
                    total_bytes: 0,
                    started_monotonic: Duration::ZERO,
                },
            },
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
