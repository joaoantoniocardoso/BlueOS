//! Recording library Block: catalog State, delete Command, rescan timers and scan IO.

#![no_std]
#![expect(
    clippy::pub_use,
    reason = "the library Block re-exports timestamp parsing and command rules at the crate root"
)]

extern crate alloc;

mod command_rules;
mod timestamp;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{error::Error, fmt, time::Duration};

use blueos_domain::{Effect, IoError, Now, Outcome};
use blueos_recorder_paths::RecordingRelativePath;

pub use command_rules::{
    DELETE_RECORDING, RecordingCommandContext, allowed_operations, delete_recording_rejection,
};
pub use timestamp::created_unix_seconds_from_filename;

// ponytail: timer rescan; switch to inotify if external writers or large folders make it costly.
/// How often the library rescans the recordings folder while idle.
pub const RESCAN_INTERVAL: Duration = Duration::from_secs(5);

/// In-memory library Block state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Library {
    entries: Vec<RecordingFileEntry>,
    scanned: BTreeMap<String, ScannedRecording>,
    deleting: BTreeSet<String>,
}

/// Commands handled by the library Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryRequest {
    /// Remove a recording from disk.
    DeleteRecording {
        /// Syntax-validated relative path.
        path: RecordingRelativePath,
    },
}

/// Why a library Command was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LibraryRejection(&'static str);

type LibraryOutcome =
    Outcome<InfallibleLibraryEvent, LibraryTick, LibraryIoRequest, LibraryTimerKey>;

/// Marker for library Block events (this Block publishes State only).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InfallibleLibraryEvent {}

/// Result of a library IO request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryIoResult {
    /// A scan finished and replaced the in-memory catalog.
    ScanCompleted {
        /// Files seen on disk (excluding footer reads for the active file).
        recordings: Vec<ScannedRecording>,
    },
    /// A scan failed; the published catalog is unchanged.
    ScanFailed,
    /// A delete attempt finished.
    DeleteFinished {
        /// Path that was deleted or attempted.
        path: RecordingRelativePath,
        /// Set when removal failed.
        error: Option<IoError>,
    },
}

/// Blocking IO the Kernel runs for the library Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryIoRequest {
    /// Rescan `.mcap` files under the recordings folder.
    Scan,
    /// Remove one recording file from disk.
    Delete {
        /// Validated relative path.
        path: RecordingRelativePath,
    },
}

/// Tick delivered on [`LibraryTimerKey::Rescan`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryTick {
    /// Rescan disk and refresh the published catalog when it changed.
    Rescan,
}

/// Timer key for periodic library rescans.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LibraryTimerKey {
    /// Periodic rescan while the service runs.
    Rescan,
}

/// One row in the published `library` State.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingFileEntry {
    /// Relative path (map key).
    pub path: String,
    /// Base name.
    pub name: String,
    /// Size in bytes.
    pub size_bytes: u64,
    /// Sort key from the file name or modification time.
    pub created_unix_seconds: i64,
    /// Row state for the UI.
    pub state: RecordingFileState,
    /// Command endpoint names accepted for this row.
    pub allowed_operations: Vec<String>,
}

/// Lifecycle state of one recording in the published library view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordingFileState {
    /// Being written by the recorder.
    Recording,
    /// Indexed and seekable.
    Ready,
    /// Finished without a summary.
    NeedsRepair,
    /// Repair in progress (not used until repair Commands exist).
    Repairing,
}

/// Metadata collected from disk during a library scan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScannedRecording {
    /// Path relative to the recordings folder.
    pub relative_path: String,
    /// Base file name.
    pub name: String,
    /// Size in bytes at scan time.
    pub size_bytes: u64,
    /// Modification time as Unix seconds.
    pub modified_unix_seconds: i64,
    /// Whether the MCAP summary is present.
    pub indexed: bool,
}

impl fmt::Display for LibraryRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Error for LibraryRejection {}

impl Library {
    /// Published catalog, newest first.
    pub fn entries(&self) -> &[RecordingFileEntry] {
        &self.entries
    }

    /// Effects to run when the library Block is first wired (initial scan only).
    pub fn initial_effects() -> Vec<Effect<LibraryTick, LibraryIoRequest, LibraryTimerKey>> {
        vec![Effect::Io(LibraryIoRequest::Scan)]
    }

    /// Handles a client Command.
    pub fn handle_request(
        &mut self,
        request: LibraryRequest,
        active_recording_relative_path: Option<&str>,
        _now: Now,
    ) -> LibraryOutcome {
        match request {
            LibraryRequest::DeleteRecording { path } => {
                let relative = path.as_str();
                let context = command_context(
                    relative,
                    active_recording_relative_path,
                    self.scanned.get(relative),
                    self.deleting.contains(relative),
                );
                if let Some(reason) = delete_recording_rejection(&context) {
                    return Outcome::reject(LibraryRejection(reason));
                }
                self.deleting.insert(relative.to_string());
                rebuild_entries(self, active_recording_relative_path);
                Outcome::Applied {
                    events: vec![],
                    effects: vec![Effect::Io(LibraryIoRequest::Delete { path })],
                }
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
        &mut self,
        result: LibraryIoResult,
        active_recording_relative_path: Option<&str>,
        _now: Now,
    ) -> LibraryOutcome {
        match result {
            LibraryIoResult::ScanCompleted { recordings } => {
                apply_scan(self, recordings, active_recording_relative_path)
            }
            LibraryIoResult::ScanFailed => finish_scan_cycle(),
            LibraryIoResult::DeleteFinished { path, error } => {
                let relative = path.as_str();
                self.deleting.remove(relative);
                if error.is_none() {
                    self.scanned.remove(relative);
                }
                rebuild_entries(self, active_recording_relative_path);
                Outcome::Applied {
                    events: vec![],
                    effects: vec![Effect::Io(LibraryIoRequest::Scan)],
                }
            }
        }
    }
}

fn finish_scan_cycle() -> LibraryOutcome {
    Outcome::Applied {
        events: vec![],
        effects: vec![arm_rescan_schedule()],
    }
}

fn arm_rescan_schedule() -> Effect<LibraryTick, LibraryIoRequest, LibraryTimerKey> {
    Effect::Schedule {
        after: RESCAN_INTERVAL,
        key: LibraryTimerKey::Rescan,
        command: LibraryTick::Rescan,
    }
}

fn apply_scan(
    library: &mut Library,
    recordings: Vec<ScannedRecording>,
    active_recording_relative_path: Option<&str>,
) -> LibraryOutcome {
    library.scanned.clear();
    for recording in recordings {
        library
            .scanned
            .insert(recording.relative_path.clone(), recording);
    }
    rebuild_entries(library, active_recording_relative_path);
    finish_scan_cycle()
}

fn rebuild_entries(library: &mut Library, active_recording_relative_path: Option<&str>) {
    let mut entries = Vec::new();
    for (path, scanned) in &library.scanned {
        let state =
            derive_recording_file_state(path, active_recording_relative_path, scanned.indexed);
        let context = command_context(
            path,
            active_recording_relative_path,
            Some(scanned),
            library.deleting.contains(path),
        );
        let created_unix_seconds =
            created_unix_seconds_from_filename(&scanned.name, scanned.modified_unix_seconds);
        entries.push(RecordingFileEntry {
            path: path.clone(),
            name: scanned.name.clone(),
            size_bytes: scanned.size_bytes,
            created_unix_seconds,
            state,
            allowed_operations: allowed_operations(&context),
        });
    }
    entries.sort_by_key(|entry| core::cmp::Reverse(entry.created_unix_seconds));
    library.entries = entries;
}

fn command_context<'a>(
    relative_path: &'a str,
    active_recording_relative_path: Option<&'a str>,
    scanned: Option<&'a ScannedRecording>,
    deleting: bool,
) -> RecordingCommandContext<'a> {
    RecordingCommandContext {
        relative_path,
        active_recording_relative_path,
        in_library: scanned.is_some(),
        deleting,
    }
}

/// Maps scan and in-flight state to the state shown in the library.
pub fn derive_recording_file_state(
    relative_path: &str,
    active_recording_relative_path: Option<&str>,
    indexed: bool,
) -> RecordingFileState {
    if active_recording_relative_path == Some(relative_path) {
        return RecordingFileState::Recording;
    }
    if indexed {
        return RecordingFileState::Ready;
    }
    RecordingFileState::NeedsRepair
}

#[cfg(test)]
mod tests;
