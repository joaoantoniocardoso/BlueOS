mod entries;
mod starts;

use alloc::{string::String, vec};

pub(crate) use entries::{command_context, finish_scan_cycle, rebuild_entries, take_operation};
pub(crate) use starts::{apply_scan, run_delete_start};
pub use starts::{start_repair, start_snapshot};

use blueos_domain::{Effect, Now, Outcome};
use blueos_jobs::JobId;
use blueos_recorder_paths::RecordingRelativePath;

use crate::repair_recording_rejection;
use crate::state::Library;
use crate::types::{
    LibraryIoRequest, LibraryIoResult, LibraryOutcome, LibraryRequest, LibraryTick, LibraryTimerKey,
};

/// Active recording path and clock readings shared by catalog commands.
pub(crate) struct LibraryCatalogScope<'a> {
    pub active_recording_relative_path: Option<&'a str>,
    pub now: Now,
}

pub(crate) struct SnapshotCommand {
    pub path: RecordingRelativePath,
    pub output_path: String,
    pub job_id: JobId,
}

pub(crate) struct RepairCommand {
    pub path: RecordingRelativePath,
    pub job_id: JobId,
}

pub(crate) struct DeleteCommand {
    pub path: RecordingRelativePath,
    pub job_id: JobId,
}

/// Inputs for [`start_snapshot`].
pub struct SnapshotStartSpec<'a> {
    /// Recording to snapshot.
    pub path: RecordingRelativePath,
    /// Indexed copy path.
    pub output_path: String,
    /// Job running the snapshot.
    pub job_id: JobId,
    /// Active recording path, if any.
    pub active_recording_relative_path: Option<&'a str>,
    /// Current time.
    pub now: Now,
}

/// Inputs for [`start_repair`].
pub struct RepairStartSpec<'a> {
    /// Recording to repair.
    pub path: RecordingRelativePath,
    /// Job running the repair.
    pub job_id: JobId,
    /// Active recording path, if any.
    pub active_recording_relative_path: Option<&'a str>,
    /// Current time.
    pub now: Now,
}

/// Effects to run when the library Block is first wired (initial scan only).
pub fn initial_effects() -> alloc::vec::Vec<Effect<LibraryTick, LibraryIoRequest, LibraryTimerKey>>
{
    alloc::vec![Effect::Io(LibraryIoRequest::Scan)]
}

/// Why repair would be rejected for `path`, if at all.
pub fn repair_rejection_for_path(
    library: &Library,
    path: &str,
    active_recording_relative_path: Option<&str>,
    now: Now,
) -> Option<&'static str> {
    let scope = LibraryCatalogScope {
        active_recording_relative_path,
        now,
    };
    let context = command_context(library, path, &scope);
    repair_recording_rejection(&context)
}

/// Handles a client Command.
pub fn handle_library_request(
    library: &mut Library,
    request: LibraryRequest,
    active_recording_relative_path: Option<&str>,
    now: Now,
) -> LibraryOutcome {
    let scope = LibraryCatalogScope {
        active_recording_relative_path,
        now,
    };
    match request {
        LibraryRequest::DeleteRecording { path, job_id } => {
            run_delete_start(library, DeleteCommand { path, job_id }, scope)
        }
    }
}

/// Handles a timer Tick.
pub fn handle_tick(tick: LibraryTick) -> LibraryOutcome {
    match tick {
        LibraryTick::Rescan => Outcome::Applied {
            events: vec![],
            effects: vec![Effect::Io(LibraryIoRequest::Scan)],
        },
    }
}

/// Handles IO results from the Kernel.
pub fn handle_io_result(
    library: &mut Library,
    result: LibraryIoResult,
    active_recording_relative_path: Option<&str>,
    now: Now,
) -> LibraryOutcome {
    let scope = LibraryCatalogScope {
        active_recording_relative_path,
        now,
    };
    match result {
        LibraryIoResult::ScanCompleted { recordings } => apply_scan(library, recordings, scope),
        LibraryIoResult::ScanFailed => finish_scan_cycle(),
        LibraryIoResult::DeleteFinished { path, error, .. } => {
            let relative = path.as_str();
            library.work.deleting.remove(relative);
            if error.is_none() {
                library.catalog.scanned.remove(relative);
            }
            rebuild_entries(library, &scope);
            Outcome::Applied {
                events: vec![],
                effects: vec![Effect::Io(LibraryIoRequest::Scan)],
            }
        }
    }
}
