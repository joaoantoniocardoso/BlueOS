//! Catalog row rebuild and entry projection.

use alloc::{string::String, string::ToString, vec, vec::Vec};

use blueos_domain::{Effect, Outcome};
use blueos_jobs::JobId;

use crate::{
    allowed_operations,
    command_rules::RecordingCommandContext,
    created_unix_seconds_from_filename, derive_recording_file_state,
    state::Library,
    types::{
        LibraryIoRequest, LibraryOperation, LibraryOutcome, LibraryTick, LibraryTimerKey,
        RESCAN_INTERVAL, RecordingFileEntry, RecordingFileState, RepairProgress, ScannedRecording,
    },
};

use super::LibraryCatalogScope;

struct RecordingFileEntryParts<'a> {
    path: &'a str,
    scanned: &'a ScannedRecording,
    created_unix_seconds: i64,
    state: RecordingFileState,
    repair_bytes_processed: u64,
    repair_total_bytes: u64,
    repair_bytes_per_second: f64,
    repair_error: String,
    context: &'a RecordingCommandContext<'a>,
    repair_job_id: Option<JobId>,
}

pub(crate) fn rebuild_entries(library: &mut Library, scope: &LibraryCatalogScope<'_>) {
    let mut entries = Vec::new();
    for (path, scanned) in &library.catalog.scanned {
        let repairing = running_repair(&library.work.queue.operations, path);
        let state = derive_recording_file_state(
            path,
            scope.active_recording_relative_path,
            repairing.is_some(),
            scanned.indexed,
        );
        let context = command_context(library, path, scope);
        let created_unix_seconds =
            created_unix_seconds_from_filename(&scanned.name, scanned.modified_unix_seconds);
        let (repair_bytes_processed, repair_total_bytes, repair_bytes_per_second) =
            if let Some((_job_id, progress)) = repairing {
                let elapsed = scope
                    .now
                    .monotonic
                    .saturating_sub(progress.started_monotonic)
                    .as_secs_f64()
                    .max(1.0);
                (
                    progress.bytes_processed,
                    progress.total_bytes,
                    progress.bytes_processed as f64 / elapsed,
                )
            } else {
                (0, 0, 0.0)
            };
        let repair_error = library
            .work
            .queue
            .repair_errors
            .get(path)
            .cloned()
            .unwrap_or_default();
        let parts = RecordingFileEntryParts {
            path,
            scanned,
            created_unix_seconds,
            state,
            repair_bytes_processed,
            repair_total_bytes,
            repair_bytes_per_second,
            repair_error,
            context: &context,
            repair_job_id: repairing.map(|(job_id, _progress)| job_id),
        };
        entries.push(recording_file_entry_from_parts(parts));
    }
    entries.sort_by_key(|entry| core::cmp::Reverse(entry.created_unix_seconds));
    library.catalog.entries = entries;
    library.catalog.entries_rebuilt_monotonic = scope.now.monotonic;
}

pub(crate) fn command_context<'a>(
    library: &'a Library,
    relative_path: &'a str,
    scope: &'a LibraryCatalogScope<'a>,
) -> RecordingCommandContext<'a> {
    let scanned = library.catalog.scanned.get(relative_path);
    RecordingCommandContext {
        relative_path,
        active_recording_relative_path: scope.active_recording_relative_path,
        in_library: scanned.is_some(),
        deleting: library.work.deleting.contains(relative_path),
        repairing: running_repair(&library.work.queue.operations, relative_path).is_some(),
        snapshotting: library.work.queue.operations.iter().any(|operation| {
            matches!(operation, LibraryOperation::Snapshot { path, .. } if path.as_str() == relative_path)
        }),
        indexed: scanned.is_some_and(|recording| recording.indexed),
        not_mcap: scanned.is_some_and(|recording| {
            library.work.queue.not_mcap.get(relative_path)
                == Some(&(recording.size_bytes, recording.modified_unix_seconds))
        }),
        modified_unix_seconds: scanned.map_or(0, |recording| recording.modified_unix_seconds),
        now: scope.now,
    }
}

pub(crate) fn running_repair<'a>(
    operations: &'a [LibraryOperation],
    relative_path: &str,
) -> Option<(JobId, &'a RepairProgress)> {
    operations.iter().find_map(|operation| match operation {
        LibraryOperation::Repair {
            path,
            job_id,
            progress,
        } if path.as_str() == relative_path => Some((*job_id, progress)),
        LibraryOperation::Repair { .. } | LibraryOperation::Snapshot { .. } => None,
    })
}

pub(crate) fn take_operation(
    operations: &mut Vec<LibraryOperation>,
    matches: impl Fn(&LibraryOperation) -> bool,
) -> Option<LibraryOperation> {
    let index = operations.iter().position(matches)?;
    Some(operations.remove(index))
}

pub(crate) fn finish_scan_cycle() -> LibraryOutcome {
    Outcome::Applied {
        events: vec![],
        effects: vec![arm_rescan_schedule()],
    }
}

fn recording_file_entry_from_parts(parts: RecordingFileEntryParts<'_>) -> RecordingFileEntry {
    RecordingFileEntry {
        path: parts.path.to_string(),
        name: parts.scanned.name.clone(),
        size_bytes: parts.scanned.size_bytes,
        created_unix_seconds: parts.created_unix_seconds,
        state: parts.state,
        repair_bytes_processed: parts.repair_bytes_processed,
        repair_total_bytes: parts.repair_total_bytes,
        repair_bytes_per_second: parts.repair_bytes_per_second,
        repair_error: parts.repair_error,
        allowed_operations: allowed_operations(parts.context),
        repair_job_id: parts.repair_job_id,
    }
}

fn arm_rescan_schedule() -> Effect<LibraryTick, LibraryIoRequest, LibraryTimerKey> {
    Effect::Schedule {
        after: RESCAN_INTERVAL,
        key: LibraryTimerKey::Rescan,
        command: LibraryTick::Rescan,
    }
}
