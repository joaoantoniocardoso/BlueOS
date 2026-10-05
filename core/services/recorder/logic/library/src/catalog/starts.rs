//! Snapshot, repair, and delete start planning and enqueue.

use alloc::{string::ToString, vec, vec::Vec};

use blueos_domain::{Effect, Outcome};

use crate::{
    delete_recording_rejection, repair_recording_rejection, snapshot_recording_rejection,
    state::Library,
    types::{
        LibraryIoRequest, LibraryOperation, LibraryOutcome, LibraryRejection, RepairProgress,
        ScannedRecording,
    },
};

use super::{
    DeleteCommand, LibraryCatalogScope, RepairCommand, RepairStartSpec, SnapshotCommand,
    SnapshotStartSpec,
    entries::{command_context, rebuild_entries},
};

enum SnapshotStartPlan {
    Rejected(&'static str),
    Proceed,
}

enum RepairStartPlan {
    Rejected(LibraryRejection),
    Enqueue(u64),
}

enum DeleteStartPlan {
    Rejected(&'static str),
    Proceed,
}

pub(crate) fn apply_scan(
    library: &mut Library,
    recordings: Vec<ScannedRecording>,
    scope: LibraryCatalogScope<'_>,
) -> LibraryOutcome {
    library.catalog.scanned = recordings
        .into_iter()
        .map(|recording| {
            let relative_path = recording.relative_path.clone();
            (relative_path, recording)
        })
        .collect();
    rebuild_entries(library, &scope);
    super::entries::finish_scan_cycle()
}

pub(crate) fn run_delete_start(
    library: &mut Library,
    command: DeleteCommand,
    scope: LibraryCatalogScope<'_>,
) -> LibraryOutcome {
    let plan = plan_delete_start(library, &command, &scope);
    apply_delete_start_plan(library, command, scope, plan)
}

/// Starts a snapshot after the Domain started its Job.
pub fn start_snapshot(library: &mut Library, spec: SnapshotStartSpec<'_>) -> LibraryOutcome {
    let scope = LibraryCatalogScope {
        active_recording_relative_path: spec.active_recording_relative_path,
        now: spec.now,
    };
    let command = SnapshotCommand {
        path: spec.path,
        output_path: spec.output_path,
        job_id: spec.job_id,
    };
    let plan = plan_snapshot_start(library, &command, &scope);
    apply_snapshot_start_plan(library, command, scope, plan)
}

/// Starts repair after the Domain started its Job.
pub fn start_repair(library: &mut Library, spec: RepairStartSpec<'_>) -> LibraryOutcome {
    let scope = LibraryCatalogScope {
        active_recording_relative_path: spec.active_recording_relative_path,
        now: spec.now,
    };
    let command = RepairCommand {
        path: spec.path,
        job_id: spec.job_id,
    };
    let plan = plan_repair_start(library, &command, &scope);
    apply_repair_start_plan(library, command, &scope, plan)
}

fn plan_snapshot_start(
    library: &Library,
    command: &SnapshotCommand,
    scope: &LibraryCatalogScope<'_>,
) -> SnapshotStartPlan {
    match snapshot_start_rejection(library, command, scope) {
        Some(reason) => SnapshotStartPlan::Rejected(reason),
        None => SnapshotStartPlan::Proceed,
    }
}

fn apply_snapshot_start_plan(
    library: &mut Library,
    command: SnapshotCommand,
    scope: LibraryCatalogScope<'_>,
    plan: SnapshotStartPlan,
) -> LibraryOutcome {
    match plan {
        SnapshotStartPlan::Rejected(reason) => Outcome::reject(LibraryRejection::new(reason)),
        SnapshotStartPlan::Proceed => snapshot_enqueue(library, command, scope),
    }
}

fn plan_repair_start(
    library: &Library,
    command: &RepairCommand,
    scope: &LibraryCatalogScope<'_>,
) -> RepairStartPlan {
    if let Some(reason) = repair_start_rejection(library, command, scope) {
        return RepairStartPlan::Rejected(LibraryRejection::new(reason));
    }
    match repair_total_bytes(library, command.path.as_str()) {
        Ok(total_bytes) => RepairStartPlan::Enqueue(total_bytes),
        Err(rejection) => RepairStartPlan::Rejected(rejection),
    }
}

fn apply_repair_start_plan(
    library: &mut Library,
    command: RepairCommand,
    scope: &LibraryCatalogScope<'_>,
    plan: RepairStartPlan,
) -> LibraryOutcome {
    match plan {
        RepairStartPlan::Rejected(rejection) => Outcome::reject(rejection),
        RepairStartPlan::Enqueue(total_bytes) => {
            repair_enqueue(library, command, scope, total_bytes)
        }
    }
}

fn plan_delete_start(
    library: &Library,
    command: &DeleteCommand,
    scope: &LibraryCatalogScope<'_>,
) -> DeleteStartPlan {
    match delete_start_rejection(library, command, scope) {
        Some(reason) => DeleteStartPlan::Rejected(reason),
        None => DeleteStartPlan::Proceed,
    }
}

fn apply_delete_start_plan(
    library: &mut Library,
    command: DeleteCommand,
    scope: LibraryCatalogScope<'_>,
    plan: DeleteStartPlan,
) -> LibraryOutcome {
    match plan {
        DeleteStartPlan::Rejected(reason) => Outcome::reject(LibraryRejection::new(reason)),
        DeleteStartPlan::Proceed => delete_enqueue(library, command, scope),
    }
}

fn snapshot_start_rejection(
    library: &Library,
    command: &SnapshotCommand,
    scope: &LibraryCatalogScope<'_>,
) -> Option<&'static str> {
    let context = command_context(library, command.path.as_str(), scope);
    snapshot_recording_rejection(&context)
}

fn repair_start_rejection(
    library: &Library,
    command: &RepairCommand,
    scope: &LibraryCatalogScope<'_>,
) -> Option<&'static str> {
    let context = command_context(library, command.path.as_str(), scope);
    repair_recording_rejection(&context)
}

fn delete_start_rejection(
    library: &Library,
    command: &DeleteCommand,
    scope: &LibraryCatalogScope<'_>,
) -> Option<&'static str> {
    let context = command_context(library, command.path.as_str(), scope);
    delete_recording_rejection(&context)
}

fn snapshot_enqueue(
    library: &mut Library,
    command: SnapshotCommand,
    scope: LibraryCatalogScope<'_>,
) -> LibraryOutcome {
    library
        .work
        .queue
        .operations
        .push(LibraryOperation::Snapshot {
            path: command.path,
            output_path: command.output_path,
            job_id: command.job_id,
        });
    rebuild_entries(library, &scope);
    Outcome::Applied {
        events: vec![],
        effects: vec![],
    }
}

fn repair_enqueue(
    library: &mut Library,
    command: RepairCommand,
    scope: &LibraryCatalogScope<'_>,
    total_bytes: u64,
) -> LibraryOutcome {
    let progress = RepairProgress {
        bytes_processed: 0,
        total_bytes,
        started_monotonic: scope.now.monotonic,
    };
    library
        .work
        .queue
        .operations
        .push(LibraryOperation::Repair {
            path: command.path,
            job_id: command.job_id,
            progress,
        });
    rebuild_entries(library, scope);
    Outcome::Applied {
        events: vec![],
        effects: vec![],
    }
}

fn repair_total_bytes(library: &Library, relative_path: &str) -> Result<u64, LibraryRejection> {
    library
        .catalog
        .scanned
        .get(relative_path)
        .map(|scanned| scanned.size_bytes)
        .ok_or(LibraryRejection::new("Recording is not in the library."))
}

fn delete_enqueue(
    library: &mut Library,
    command: DeleteCommand,
    scope: LibraryCatalogScope<'_>,
) -> LibraryOutcome {
    let relative = command.path.as_str();
    library.work.deleting.insert(relative.to_string());
    rebuild_entries(library, &scope);
    Outcome::Applied {
        events: vec![],
        effects: vec![Effect::Io(LibraryIoRequest::Delete {
            path: command.path,
            job_id: command.job_id,
        })],
    }
}
