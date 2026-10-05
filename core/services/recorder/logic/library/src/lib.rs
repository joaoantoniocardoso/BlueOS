//! Recording library Block: catalog State, the repairs and snapshots it wants, delete, rescan timers and scan IO.

#![no_std]
#![expect(
    clippy::pub_use,
    reason = "the library Block re-exports timestamp parsing and command rules at the crate root"
)]

extern crate alloc;

mod catalog;
mod command_rules;
mod observed;
mod snapshot;
mod state;
mod timestamp;
mod types;

pub use catalog::{
    RepairStartSpec, SnapshotStartSpec, handle_io_result, handle_library_request, handle_tick,
    initial_effects, repair_rejection_for_path, start_repair, start_snapshot,
};
pub use command_rules::{
    CANCEL_JOB, DELETE_RECORDING, RECENTLY_WRITTEN_DELAY, REPAIR_RECORDING,
    RecordingCommandContext, SNAPSHOT_RECORDING, allowed_operations, cancel_repair_rejection,
    delete_recording_rejection, repair_recording_rejection, snapshot_recording_rejection,
};
pub use observed::handle_observed_fact;
pub use snapshot::snapshot_output_relative_path;
pub use state::{Library, LibraryCatalog, LibraryWork};
pub use timestamp::created_unix_seconds_from_filename;
pub use types::{
    LibraryIoRequest, LibraryIoResult, LibraryObservedFact, LibraryOperation, LibraryOutcome,
    LibraryRejection, LibraryRepairOutcome, LibraryRepairProgress, LibraryRequest,
    LibrarySnapshotOutcome, LibraryTick, LibraryTimerKey, REPAIR_PROGRESS_PUBLISH_INTERVAL,
    RESCAN_INTERVAL, RecordingContents, RecordingFileEntry, RecordingFileState, RepairFailure,
    RepairProgress, ScannedRecording,
};

// ponytail: timer rescan; switch to inotify if external writers or large folders make it costly.

/// Maps scan and in-flight state to the state shown in the library.
pub fn derive_recording_file_state(
    relative_path: &str,
    active_recording_relative_path: Option<&str>,
    repairing: bool,
    indexed: bool,
) -> RecordingFileState {
    if repairing {
        return RecordingFileState::Repairing;
    }
    if active_recording_relative_path == Some(relative_path) {
        return RecordingFileState::Recording;
    }
    if indexed {
        return RecordingFileState::Ready;
    }
    RecordingFileState::NeedsRepair
}
