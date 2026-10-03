//! Recording library Block: catalog State, repair and delete Commands, rescan timers and scan IO.

#![no_std]
#![expect(
    clippy::pub_use,
    reason = "the library Block re-exports timestamp parsing and command rules at the crate root"
)]

extern crate alloc;

mod command_rules;
mod snapshot;
mod timestamp;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{error::Error, fmt, time::Duration};

use blueos_domain::{Effect, IoError, Now, Outcome};
use blueos_jobs::JobId;
use blueos_recorder_paths::RecordingRelativePath;

pub use command_rules::{
    CANCEL_REPAIR, DELETE_RECORDING, RECENTLY_WRITTEN_DELAY, REPAIR_RECORDING,
    RecordingCommandContext, SNAPSHOT_RECORDING, allowed_operations, cancel_repair_rejection,
    delete_recording_rejection, repair_recording_rejection, snapshot_recording_rejection,
};
pub use snapshot::snapshot_output_relative_path;
pub use timestamp::created_unix_seconds_from_filename;

// ponytail: timer rescan; switch to inotify if external writers or large folders make it costly.
/// How often the library rescans the recordings folder while idle.
pub const RESCAN_INTERVAL: Duration = Duration::from_secs(5);

/// In-memory library Block state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Library {
    entries: Vec<RecordingFileEntry>,
    scanned: BTreeMap<String, ScannedRecording>,
    deleting: BTreeSet<String>,
    repairing: BTreeMap<String, RepairProgress>,
    snapshotting: BTreeMap<String, String>,
    repair_errors: BTreeMap<String, String>,
}

/// Commands handled by the library Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryRequest {
    /// Stops a running repair without touching the original file.
    CancelRepair {
        /// Syntax-validated relative path.
        path: RecordingRelativePath,
    },
    /// Remove a recording from disk.
    DeleteRecording {
        /// Syntax-validated relative path.
        path: RecordingRelativePath,
    },
}

/// Progress of one in-flight repair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepairProgress {
    /// Bytes read from the source file so far.
    pub bytes_processed: u64,
    /// Source size at repair start.
    pub total_bytes: u64,
    /// Monotonic time when repair started.
    pub started_monotonic: Duration,
    /// The Job to end when IO ends.
    pub job_id: JobId,
}

/// Events published after library work completes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryEvent {
    /// Repair, snapshot, or delete finished.
    Operation(RecordingOperationEvent),
}

/// Marker for library Block events when none apply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InfallibleLibraryEvent {}

/// Outcome of repair, snapshot, or delete work emitted to subscribers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingOperationEvent {
    /// Which operation ended.
    pub operation: RecordingOperationKind,
    /// Recording path.
    pub path: String,
    /// Snapshot copy path (empty for repair).
    pub output_path: String,
    /// Whether the work succeeded.
    pub succeeded: bool,
    /// Whether the work was cancelled.
    pub cancelled: bool,
    /// Failure reason when not cancelled.
    pub failure: RepairFailure,
}

/// Which library operation finished.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordingOperationKind {
    /// Native MCAP rewrite.
    Repair,
    /// Indexed copy of a recording still being written.
    Snapshot,
}

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
    /// A repair attempt finished.
    RepairFinished {
        /// Path that was repaired or attempted.
        path: RecordingRelativePath,
        /// How the repair ended.
        outcome: LibraryRepairOutcome,
    },
    /// A snapshot attempt finished.
    SnapshotFinished {
        /// Source recording path.
        path: RecordingRelativePath,
        /// Snapshot file path relative to the recordings folder.
        output_path: String,
        /// How the snapshot ended.
        outcome: LibrarySnapshotOutcome,
    },
}

/// How a snapshot IO request ended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibrarySnapshotOutcome {
    /// The indexed copy was written next to the source.
    Succeeded,
    /// Rewrite or rename failed.
    Failed(RepairFailure),
}

/// How a repair IO request ended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryRepairOutcome {
    /// The file was rewritten and replaced.
    Succeeded,
    /// Cancel removed the temporary file.
    Cancelled,
    /// Rewrite or replace failed.
    Failed(RepairFailure),
}

/// Typed repair failure reasons in logic (mapped to wire strings in the api crate).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepairFailure {
    /// No failure (success or cancel).
    None,
    /// Filesystem or resolve error.
    Io,
    /// MCAP rewrite failed.
    Rewrite,
    /// Replace after rewrite failed.
    Replace,
    /// Adapter reported a message (reserved for future typed mapping).
    Message(String),
}

/// Why a library Command was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LibraryRejection(&'static str);

/// Observed fact from repair IO (progress updates).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryRepairProgress {
    /// Recording being repaired.
    pub path: RecordingRelativePath,
    /// Bytes read from the source.
    pub bytes_processed: u64,
    /// Total bytes to read.
    pub total_bytes: u64,
}

type LibraryOutcome = Outcome<LibraryEvent, LibraryTick, LibraryIoRequest, LibraryTimerKey>;

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
    /// Rewrite one recording through a `.recover` file.
    Repair {
        /// Validated relative path.
        path: RecordingRelativePath,
    },
    /// Signal a running repair to stop and remove its temporary file.
    CancelRepair {
        /// Validated relative path.
        path: RecordingRelativePath,
    },
    /// Write an indexed copy next to a recording.
    Snapshot {
        /// Validated source path.
        path: RecordingRelativePath,
        /// Relative path for the finished snapshot file.
        output_path: String,
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
#[derive(Clone, Debug, PartialEq)]
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
    /// Repair progress while [`RecordingFileState::Repairing`].
    pub repair_bytes_processed: u64,
    /// Total bytes to read for the active repair.
    pub repair_total_bytes: u64,
    /// Smoothed repair throughput for the UI.
    pub repair_bytes_per_second: f64,
    /// Last repair failure message for the UI.
    pub repair_error: String,
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
    /// Repair in progress.
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

impl LibraryRejection {
    /// Builds a rejection reason the Domain can return to clients.
    pub const fn new(reason: &'static str) -> Self {
        Self(reason)
    }
}

impl Library {
    /// Published catalog, newest first.
    pub fn entries(&self) -> &[RecordingFileEntry] {
        &self.entries
    }

    /// Effects to run when the library Block is first wired (initial scan only).
    pub fn initial_effects() -> Vec<Effect<LibraryTick, LibraryIoRequest, LibraryTimerKey>> {
        vec![Effect::Io(LibraryIoRequest::Scan)]
    }

    /// Why repair would be rejected for `path`, if at all.
    pub fn repair_rejection_for_path(
        &self,
        path: &str,
        active_recording_relative_path: Option<&str>,
        now: Now,
    ) -> Option<&'static str> {
        let context = command_context(self, path, active_recording_relative_path, now);
        repair_recording_rejection(&context)
    }

    /// Starts a snapshot after the Domain accepted the Command.
    pub fn start_snapshot(
        &mut self,
        path: RecordingRelativePath,
        output_path: String,
        active_recording_relative_path: Option<&str>,
        now: Now,
    ) -> LibraryOutcome {
        let relative = path.as_str();
        let context = command_context(self, relative, active_recording_relative_path, now);
        if let Some(reason) = snapshot_recording_rejection(&context) {
            return Outcome::reject(LibraryRejection(reason));
        }
        self.snapshotting
            .insert(relative.to_string(), output_path.clone());
        rebuild_entries(self, active_recording_relative_path, now);
        Outcome::Applied {
            events: vec![],
            effects: vec![Effect::Io(LibraryIoRequest::Snapshot { path, output_path })],
        }
    }

    /// Starts repair after the Domain started its Job.
    pub fn start_repair(
        &mut self,
        path: RecordingRelativePath,
        job_id: JobId,
        active_recording_relative_path: Option<&str>,
        now: Now,
    ) -> LibraryOutcome {
        let relative = path.as_str();
        let context = command_context(self, relative, active_recording_relative_path, now);
        if let Some(reason) = repair_recording_rejection(&context) {
            return Outcome::reject(LibraryRejection(reason));
        }
        let scanned = self.scanned.get(relative).expect("validated in library");
        self.repairing.insert(
            relative.to_string(),
            RepairProgress {
                bytes_processed: 0,
                total_bytes: scanned.size_bytes,
                started_monotonic: now.monotonic,
                job_id,
            },
        );
        rebuild_entries(self, active_recording_relative_path, now);
        Outcome::Applied {
            events: vec![],
            effects: vec![Effect::Io(LibraryIoRequest::Repair { path })],
        }
    }

    /// Handles a client Command.
    pub fn handle_request(
        &mut self,
        request: LibraryRequest,
        active_recording_relative_path: Option<&str>,
        now: Now,
    ) -> LibraryOutcome {
        match request {
            LibraryRequest::CancelRepair { path } => {
                let relative = path.as_str();
                let context = command_context(self, relative, active_recording_relative_path, now);
                if let Some(reason) = cancel_repair_rejection(&context) {
                    return Outcome::reject(LibraryRejection(reason));
                }
                Outcome::Applied {
                    events: vec![],
                    effects: vec![Effect::Io(LibraryIoRequest::CancelRepair { path })],
                }
            }
            LibraryRequest::DeleteRecording { path } => {
                let relative = path.as_str();
                let context = command_context(self, relative, active_recording_relative_path, now);
                if let Some(reason) = delete_recording_rejection(&context) {
                    return Outcome::reject(LibraryRejection(reason));
                }
                self.deleting.insert(relative.to_string());
                rebuild_entries(self, active_recording_relative_path, now);
                Outcome::Applied {
                    events: vec![],
                    effects: vec![Effect::Io(LibraryIoRequest::Delete { path })],
                }
            }
        }
    }

    /// Applies a repair progress Observed fact.
    pub fn handle_repair_progress(
        &mut self,
        progress: LibraryRepairProgress,
        active_recording_relative_path: Option<&str>,
        now: Now,
    ) -> LibraryOutcome {
        let relative = progress.path.as_str();
        if let Some(state) = self.repairing.get_mut(relative) {
            state.bytes_processed = progress.bytes_processed.min(progress.total_bytes);
            state.total_bytes = progress.total_bytes;
        }
        rebuild_entries(self, active_recording_relative_path, now);
        Outcome::Applied {
            events: vec![],
            effects: vec![],
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
        now: Now,
    ) -> LibraryOutcome {
        match result {
            LibraryIoResult::ScanCompleted { recordings } => {
                apply_scan(self, recordings, active_recording_relative_path, now)
            }
            LibraryIoResult::ScanFailed => finish_scan_cycle(),
            LibraryIoResult::DeleteFinished { path, error } => {
                let relative = path.as_str();
                self.deleting.remove(relative);
                if error.is_none() {
                    self.scanned.remove(relative);
                }
                rebuild_entries(self, active_recording_relative_path, now);
                Outcome::Applied {
                    events: vec![],
                    effects: vec![Effect::Io(LibraryIoRequest::Scan)],
                }
            }
            LibraryIoResult::SnapshotFinished {
                path,
                output_path,
                outcome,
            } => {
                let relative = path.as_str();
                self.snapshotting.remove(relative);
                let (succeeded, failure) = match outcome {
                    LibrarySnapshotOutcome::Succeeded => (true, RepairFailure::None),
                    LibrarySnapshotOutcome::Failed(failure) => (false, failure),
                };
                rebuild_entries(self, active_recording_relative_path, now);
                let event = RecordingOperationEvent {
                    operation: RecordingOperationKind::Snapshot,
                    path: relative.to_string(),
                    output_path,
                    succeeded,
                    cancelled: false,
                    failure,
                };
                Outcome::Applied {
                    events: vec![LibraryEvent::Operation(event)],
                    effects: vec![Effect::Io(LibraryIoRequest::Scan)],
                }
            }
            LibraryIoResult::RepairFinished { path, outcome } => {
                let relative = path.as_str();
                self.repairing.remove(relative);
                let (succeeded, cancelled, failure) = match outcome {
                    LibraryRepairOutcome::Succeeded => (true, false, RepairFailure::None),
                    LibraryRepairOutcome::Cancelled => (false, true, RepairFailure::None),
                    LibraryRepairOutcome::Failed(failure) => (false, false, failure),
                };
                if succeeded {
                    self.repair_errors.remove(relative);
                } else if !cancelled {
                    self.repair_errors
                        .insert(relative.to_string(), repair_failure_message(&failure));
                }
                rebuild_entries(self, active_recording_relative_path, now);
                let event = RecordingOperationEvent {
                    operation: RecordingOperationKind::Repair,
                    path: relative.to_string(),
                    output_path: String::new(),
                    succeeded,
                    cancelled,
                    failure: if cancelled {
                        RepairFailure::None
                    } else {
                        failure
                    },
                };
                Outcome::Applied {
                    events: vec![LibraryEvent::Operation(event)],
                    effects: vec![Effect::Io(LibraryIoRequest::Scan)],
                }
            }
        }
    }

    /// Job id for an in-flight repair, if any.
    pub fn repair_job_id(&self, path: &str) -> Option<JobId> {
        self.repairing.get(path).map(|progress| progress.job_id)
    }
}

fn repair_failure_message(failure: &RepairFailure) -> String {
    match failure {
        RepairFailure::None => String::new(),
        RepairFailure::Io => "Filesystem operation failed.".into(),
        RepairFailure::Rewrite => "MCAP rewrite failed.".into(),
        RepairFailure::Replace => "Could not replace the recording file.".into(),
        RepairFailure::Message(message) => message.clone(),
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
    now: Now,
) -> LibraryOutcome {
    library.scanned.clear();
    for recording in recordings {
        library
            .scanned
            .insert(recording.relative_path.clone(), recording);
    }
    rebuild_entries(library, active_recording_relative_path, now);
    finish_scan_cycle()
}

fn rebuild_entries(library: &mut Library, active_recording_relative_path: Option<&str>, now: Now) {
    let mut entries = Vec::new();
    for (path, scanned) in &library.scanned {
        let repairing = library.repairing.get(path);
        let state = derive_recording_file_state(
            path,
            active_recording_relative_path,
            repairing.is_some(),
            scanned.indexed,
        );
        let context = command_context(library, path, active_recording_relative_path, now);
        let created_unix_seconds =
            created_unix_seconds_from_filename(&scanned.name, scanned.modified_unix_seconds);
        let (repair_bytes_processed, repair_total_bytes, repair_bytes_per_second) =
            if let Some(progress) = repairing {
                let elapsed = now
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
        let repair_error = library.repair_errors.get(path).cloned().unwrap_or_default();
        entries.push(RecordingFileEntry {
            path: path.clone(),
            name: scanned.name.clone(),
            size_bytes: scanned.size_bytes,
            created_unix_seconds,
            state,
            repair_bytes_processed,
            repair_total_bytes,
            repair_bytes_per_second,
            repair_error,
            allowed_operations: allowed_operations(&context),
        });
    }
    entries.sort_by_key(|entry| core::cmp::Reverse(entry.created_unix_seconds));
    library.entries = entries;
}

fn command_context<'a>(
    library: &'a Library,
    relative_path: &'a str,
    active_recording_relative_path: Option<&'a str>,
    now: Now,
) -> RecordingCommandContext<'a> {
    let scanned = library.scanned.get(relative_path);
    RecordingCommandContext {
        relative_path,
        active_recording_relative_path,
        in_library: scanned.is_some(),
        deleting: library.deleting.contains(relative_path),
        repairing: library.repairing.contains_key(relative_path),
        snapshotting: library.snapshotting.contains_key(relative_path),
        indexed: scanned.is_some_and(|recording| recording.indexed),
        modified_unix_seconds: scanned.map_or(0, |recording| recording.modified_unix_seconds),
        now,
    }
}

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

#[cfg(test)]
mod tests;
