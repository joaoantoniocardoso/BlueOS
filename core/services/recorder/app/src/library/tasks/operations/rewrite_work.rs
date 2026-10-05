//! Blocking repair and snapshot rewrites for library operations.

use core::sync::atomic::{AtomicBool, Ordering};
use std::{fs, io, path::Path};

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
use blueos_service::CommandSender;

use crate::context::Rewriter;

/// The fact that `operation` ended, with the outcome `repair_outcome` or `snapshot_outcome` gives it.
pub(super) fn finished_fact(
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
pub(super) fn repair(
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
pub(super) fn snapshot(
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
