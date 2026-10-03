//! Recording library Block: catalog State, the repairs and snapshots it wants, delete, rescan timers and scan IO.

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
use core::{convert::Infallible, error::Error, fmt, time::Duration};

use blueos_domain::{Effect, IoError, Now, Outcome};
use blueos_jobs::JobId;
use blueos_recorder_paths::RecordingRelativePath;

pub use command_rules::{
    CANCEL_JOB, DELETE_RECORDING, RECENTLY_WRITTEN_DELAY, REPAIR_RECORDING,
    RecordingCommandContext, SNAPSHOT_RECORDING, allowed_operations, cancel_repair_rejection,
    delete_recording_rejection, repair_recording_rejection, snapshot_recording_rejection,
};
pub use snapshot::snapshot_output_relative_path;
pub use timestamp::created_unix_seconds_from_filename;

// ponytail: timer rescan; switch to inotify if external writers or large folders make it costly.
/// How often the library rescans the recordings folder while idle.
pub const RESCAN_INTERVAL: Duration = Duration::from_secs(5);
/// How long a repair's read offset waits, after the library entries were last rebuilt, before it is published in
/// them; the Job's Feedback always has the latest one.
pub const REPAIR_PROGRESS_PUBLISH_INTERVAL: Duration = Duration::from_millis(500);

/// In-memory library Block state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Library {
    entries: Vec<RecordingFileEntry>,
    scanned: BTreeMap<String, ScannedRecording>,
    deleting: BTreeSet<String>,
    operations: Vec<LibraryOperation>,
    ended: Option<LibraryOperation>,
    repair_errors: BTreeMap<String, String>,
    /// Files whose repair failed because they are not MCAP, with the size and modification time they had.
    not_mcap: BTreeMap<String, (u64, i64)>,
    /// Monotonic time the entries were last rebuilt.
    entries_rebuilt_monotonic: Duration,
}

/// Commands handled by the library Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryRequest {
    /// Remove a recording from disk.
    DeleteRecording {
        /// Syntax-validated relative path.
        path: RecordingRelativePath,
        /// The Job the delete runs as.
        job_id: JobId,
    },
}

/// A rewrite the library wants done. The operations Task follows the list of them, runs each rewrite, and reports
/// how it ended as a [`LibraryObservedFact`] (D-27).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryOperation {
    /// Rewrite a recording in place through a `.recover` file.
    Repair {
        /// Validated relative path.
        path: RecordingRelativePath,
        /// The Job the repair runs as.
        job_id: JobId,
        /// How far the repair has read.
        progress: RepairProgress,
    },
    /// Write an indexed copy next to a recording.
    Snapshot {
        /// Validated source path.
        path: RecordingRelativePath,
        /// Relative path for the finished snapshot file.
        output_path: String,
        /// The Job the snapshot runs as.
        job_id: JobId,
    },
}

/// What the operations Task observed while running a [`LibraryOperation`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryObservedFact {
    /// A repair read further into its source.
    RepairProgress(LibraryRepairProgress),
    /// A repair ended.
    RepairFinished {
        /// Path that was repaired or attempted.
        path: RecordingRelativePath,
        /// How the repair ended.
        outcome: LibraryRepairOutcome,
    },
    /// A snapshot ended.
    SnapshotFinished {
        /// Source recording path.
        path: RecordingRelativePath,
        /// Snapshot file path relative to the recordings folder.
        output_path: String,
        /// How the snapshot ended.
        outcome: LibrarySnapshotOutcome,
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
        /// The Job the delete ran as.
        job_id: JobId,
        /// Set when removal failed.
        error: Option<IoError>,
    },
}

/// How a snapshot ended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibrarySnapshotOutcome {
    /// The indexed copy was written next to the source.
    Succeeded,
    /// Rewrite or rename failed.
    Failed(RepairFailure),
}

/// How a repair ended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryRepairOutcome {
    /// The file was rewritten and replaced.
    Succeeded,
    /// Cancel removed the temporary file.
    Cancelled,
    /// Rewrite or replace failed.
    Failed(RepairFailure),
}

/// Why a repair or snapshot failed. Its text is the reason of the Job and the error of the library row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepairFailure {
    /// Filesystem or resolve error.
    Io,
    /// MCAP rewrite failed.
    Rewrite,
    /// The file is not an MCAP recording, so repairing it again cannot succeed.
    NotMcap,
    /// Replace after rewrite failed.
    Replace,
    /// Adapter reported a message (reserved for future typed mapping).
    Message(String),
}

/// Why a library Command was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LibraryRejection(&'static str);

/// How far a repair has read into its source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryRepairProgress {
    /// Recording being repaired.
    pub path: RecordingRelativePath,
    /// Bytes read from the source.
    pub bytes_processed: u64,
    /// Total bytes to read.
    pub total_bytes: u64,
}

type LibraryOutcome = Outcome<Infallible, LibraryTick, LibraryIoRequest, LibraryTimerKey>;

/// Blocking IO the Kernel runs for the library Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryIoRequest {
    /// Rescan `.mcap` files under the recordings folder.
    Scan,
    /// Remove one recording file from disk.
    Delete {
        /// Validated relative path.
        path: RecordingRelativePath,
        /// The Job the delete runs as.
        job_id: JobId,
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
    /// The Job of the repair while [`RecordingFileState::Repairing`].
    pub repair_job_id: Option<JobId>,
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

impl fmt::Display for RepairFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io => formatter.write_str("Filesystem operation failed."),
            Self::Rewrite => formatter.write_str("MCAP rewrite failed."),
            Self::NotMcap => formatter.write_str("This is not an MCAP file."),
            Self::Replace => formatter.write_str("Could not replace the recording file."),
            Self::Message(message) => formatter.write_str(message),
        }
    }
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

    /// The repairs and snapshots the library wants done, in the order they started.
    pub fn operations(&self) -> &[LibraryOperation] {
        &self.operations
    }

    /// The repair or snapshot that ended last, kept until the next one ends, so its Job result can be read in the
    /// step that ended it.
    pub fn ended_operation(&self) -> Option<&LibraryOperation> {
        self.ended.as_ref()
    }

    /// How far the repair the Job `job_id` runs has read, while it runs.
    pub fn repair_progress(&self, job_id: JobId) -> Option<&RepairProgress> {
        self.operations
            .iter()
            .find_map(|operation| match operation {
                LibraryOperation::Repair {
                    job_id: running,
                    progress,
                    ..
                } if *running == job_id => Some(progress),
                LibraryOperation::Repair { .. } | LibraryOperation::Snapshot { .. } => None,
            })
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

    /// Starts a snapshot after the Domain started its Job.
    pub fn start_snapshot(
        &mut self,
        path: RecordingRelativePath,
        output_path: String,
        job_id: JobId,
        active_recording_relative_path: Option<&str>,
        now: Now,
    ) -> LibraryOutcome {
        let relative = path.as_str();
        let context = command_context(self, relative, active_recording_relative_path, now);
        if let Some(reason) = snapshot_recording_rejection(&context) {
            return Outcome::reject(LibraryRejection(reason));
        }
        self.operations.push(LibraryOperation::Snapshot {
            path,
            output_path,
            job_id,
        });
        rebuild_entries(self, active_recording_relative_path, now);
        Outcome::Applied {
            events: vec![],
            effects: vec![],
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
        let progress = RepairProgress {
            bytes_processed: 0,
            total_bytes: scanned.size_bytes,
            started_monotonic: now.monotonic,
        };
        self.operations.push(LibraryOperation::Repair {
            path,
            job_id,
            progress,
        });
        rebuild_entries(self, active_recording_relative_path, now);
        Outcome::Applied {
            events: vec![],
            effects: vec![],
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
            LibraryRequest::DeleteRecording { path, job_id } => {
                let relative = path.as_str();
                let context = command_context(self, relative, active_recording_relative_path, now);
                if let Some(reason) = delete_recording_rejection(&context) {
                    return Outcome::reject(LibraryRejection(reason));
                }
                self.deleting.insert(relative.to_string());
                rebuild_entries(self, active_recording_relative_path, now);
                Outcome::Applied {
                    events: vec![],
                    effects: vec![Effect::Io(LibraryIoRequest::Delete { path, job_id })],
                }
            }
        }
    }

    /// Applies what the operations Task observed: a repair's progress, or the end of a repair or snapshot, which
    /// removes it from [`Library::operations`].
    pub fn handle_observed_fact(
        &mut self,
        fact: LibraryObservedFact,
        active_recording_relative_path: Option<&str>,
        now: Now,
    ) -> LibraryOutcome {
        match fact {
            LibraryObservedFact::RepairProgress(observed) => {
                for operation in &mut self.operations {
                    if let LibraryOperation::Repair { path, progress, .. } = operation
                        && *path == observed.path
                    {
                        progress.bytes_processed =
                            observed.bytes_processed.min(observed.total_bytes);
                        progress.total_bytes = observed.total_bytes;
                    }
                }
                if observed.bytes_processed >= observed.total_bytes
                    || now.monotonic.saturating_sub(self.entries_rebuilt_monotonic)
                        >= REPAIR_PROGRESS_PUBLISH_INTERVAL
                {
                    rebuild_entries(self, active_recording_relative_path, now);
                }
                Outcome::Applied {
                    events: vec![],
                    effects: vec![],
                }
            }
            LibraryObservedFact::SnapshotFinished { path, .. } => {
                self.ended = take_operation(
                    &mut self.operations,
                    |operation| matches!(operation, LibraryOperation::Snapshot { path: source, .. } if *source == path),
                );
                rebuild_entries(self, active_recording_relative_path, now);
                Outcome::Applied {
                    events: vec![],
                    effects: vec![Effect::Io(LibraryIoRequest::Scan)],
                }
            }
            LibraryObservedFact::RepairFinished { path, outcome } => {
                self.ended = take_operation(
                    &mut self.operations,
                    |operation| matches!(operation, LibraryOperation::Repair { path: repaired, .. } if *repaired == path),
                );
                let relative = path.as_str();
                match outcome {
                    LibraryRepairOutcome::Succeeded => {
                        self.repair_errors.remove(relative);
                        self.not_mcap.remove(relative);
                        // The scan that follows confirms it; until then the row must not look unindexed.
                        if let Some(scanned) = self.scanned.get_mut(relative) {
                            scanned.indexed = true;
                        }
                    }
                    LibraryRepairOutcome::Cancelled => {}
                    LibraryRepairOutcome::Failed(failure) => {
                        if failure == RepairFailure::NotMcap
                            && let Some(scanned) = self.scanned.get(relative)
                        {
                            self.not_mcap.insert(
                                relative.to_string(),
                                (scanned.size_bytes, scanned.modified_unix_seconds),
                            );
                        }
                        self.repair_errors
                            .insert(relative.to_string(), failure.to_string());
                    }
                }
                rebuild_entries(self, active_recording_relative_path, now);
                Outcome::Applied {
                    events: vec![],
                    effects: vec![Effect::Io(LibraryIoRequest::Scan)],
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
        now: Now,
    ) -> LibraryOutcome {
        match result {
            LibraryIoResult::ScanCompleted { recordings } => {
                apply_scan(self, recordings, active_recording_relative_path, now)
            }
            LibraryIoResult::ScanFailed => finish_scan_cycle(),
            LibraryIoResult::DeleteFinished { path, error, .. } => {
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
        }
    }
}

impl LibraryOperation {
    /// The Job the operation runs as, which names it while its progress changes.
    pub fn job_id(&self) -> JobId {
        match self {
            Self::Repair { job_id, .. } | Self::Snapshot { job_id, .. } => *job_id,
        }
    }
}

/// Removes the first operation that `matches` accepts from `operations`, if any, and returns it.
fn take_operation(
    operations: &mut Vec<LibraryOperation>,
    matches: impl Fn(&LibraryOperation) -> bool,
) -> Option<LibraryOperation> {
    let index = operations.iter().position(matches)?;
    Some(operations.remove(index))
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
        let repairing = running_repair(&library.operations, path);
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
            if let Some((_job_id, progress)) = repairing {
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
            repair_job_id: repairing.map(|(job_id, _progress)| job_id),
        });
    }
    entries.sort_by_key(|entry| core::cmp::Reverse(entry.created_unix_seconds));
    library.entries = entries;
    library.entries_rebuilt_monotonic = now.monotonic;
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
        repairing: running_repair(&library.operations, relative_path).is_some(),
        snapshotting: library.operations.iter().any(|operation| {
            matches!(operation, LibraryOperation::Snapshot { path, .. } if path.as_str() == relative_path)
        }),
        indexed: scanned.is_some_and(|recording| recording.indexed),
        not_mcap: scanned.is_some_and(|recording| {
            library.not_mcap.get(relative_path)
                == Some(&(recording.size_bytes, recording.modified_unix_seconds))
        }),
        modified_unix_seconds: scanned.map_or(0, |recording| recording.modified_unix_seconds),
        now,
    }
}

/// The Job and progress of the repair of `relative_path` in `operations`, while it runs.
fn running_repair<'a>(
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
