use alloc::{string::ToString, vec};

use blueos_domain::{Effect, Now, Outcome};
use blueos_recorder_paths::RecordingRelativePath;

use crate::{
    catalog::{LibraryCatalogScope, rebuild_entries, take_operation},
    state::Library,
    types::{
        LibraryIoRequest, LibraryObservedFact, LibraryOperation, LibraryOutcome,
        LibraryRepairOutcome, LibraryRepairProgress, REPAIR_PROGRESS_PUBLISH_INTERVAL,
        RepairFailure,
    },
};

fn catalog_scope<'a>(
    active_recording_relative_path: Option<&'a str>,
    now: Now,
) -> LibraryCatalogScope<'a> {
    LibraryCatalogScope {
        active_recording_relative_path,
        now,
    }
}

/// Applies what the operations Task observed: repair progress or the end of a repair or snapshot.
pub fn handle_observed_fact(
    library: &mut Library,
    fact: LibraryObservedFact,
    active_recording_relative_path: Option<&str>,
    now: Now,
) -> LibraryOutcome {
    match fact {
        LibraryObservedFact::RepairProgress(observed) => {
            apply_repair_progress(library, observed, active_recording_relative_path, now)
        }
        LibraryObservedFact::SnapshotFinished { path, .. } => {
            apply_snapshot_finished(library, path, active_recording_relative_path, now)
        }
        LibraryObservedFact::RepairFinished { path, outcome } => {
            apply_repair_finished(library, path, outcome, active_recording_relative_path, now)
        }
    }
}

fn apply_repair_progress(
    library: &mut Library,
    observed: LibraryRepairProgress,
    active_recording_relative_path: Option<&str>,
    now: Now,
) -> LibraryOutcome {
    for operation in &mut library.work.queue.operations {
        if let LibraryOperation::Repair { path, progress, .. } = operation
            && *path == observed.path
        {
            progress.bytes_processed = observed.bytes_processed.min(observed.total_bytes);
            progress.total_bytes = observed.total_bytes;
        }
    }
    if observed.bytes_processed >= observed.total_bytes
        || now
            .monotonic
            .saturating_sub(library.catalog.entries_rebuilt_monotonic)
            >= REPAIR_PROGRESS_PUBLISH_INTERVAL
    {
        rebuild_entries(library, &catalog_scope(active_recording_relative_path, now));
    }
    Outcome::Applied {
        events: vec![],
        effects: vec![],
    }
}

fn apply_snapshot_finished(
    library: &mut Library,
    path: RecordingRelativePath,
    active_recording_relative_path: Option<&str>,
    now: Now,
) -> LibraryOutcome {
    library.work.queue.ended = take_operation(&mut library.work.queue.operations, |operation| {
        matches!(
            operation,
            LibraryOperation::Snapshot { path: source, .. } if *source == path
        )
    });
    rebuild_entries(library, &catalog_scope(active_recording_relative_path, now));
    Outcome::Applied {
        events: vec![],
        effects: vec![Effect::Io(LibraryIoRequest::Scan)],
    }
}

fn apply_repair_finished(
    library: &mut Library,
    path: RecordingRelativePath,
    outcome: LibraryRepairOutcome,
    active_recording_relative_path: Option<&str>,
    now: Now,
) -> LibraryOutcome {
    library.work.queue.ended = take_operation(&mut library.work.queue.operations, |operation| {
        matches!(
            operation,
            LibraryOperation::Repair { path: repaired, .. } if *repaired == path
        )
    });
    let relative = path.as_str();
    match outcome {
        LibraryRepairOutcome::Succeeded => {
            library.work.queue.repair_errors.remove(relative);
            library.work.queue.not_mcap.remove(relative);
            if let Some(scanned) = library.catalog.scanned.get_mut(relative) {
                scanned.indexed = true;
            }
        }
        LibraryRepairOutcome::Cancelled => {}
        LibraryRepairOutcome::Failed(failure) => {
            if failure == RepairFailure::NotMcap
                && let Some(scanned) = library.catalog.scanned.get(relative)
            {
                library.work.queue.not_mcap.insert(
                    relative.to_string(),
                    (scanned.size_bytes, scanned.modified_unix_seconds),
                );
            }
            library
                .work
                .queue
                .repair_errors
                .insert(relative.to_string(), failure.to_string());
        }
    }
    rebuild_entries(library, &catalog_scope(active_recording_relative_path, now));
    Outcome::Applied {
        events: vec![],
        effects: vec![Effect::Io(LibraryIoRequest::Scan)],
    }
}
