use alloc::{string::String, vec::Vec};
use core::{fmt, time::Duration};

use blueos_domain::{IoError, Outcome};
use blueos_jobs::JobId;
use blueos_recorder_paths::RecordingRelativePath;

/// How often the library rescans the recordings folder while idle.
pub const RESCAN_INTERVAL: Duration = Duration::from_secs(5);
/// How long a repair's read offset waits, after the library entries were last rebuilt, before it is published in
/// them; the Job's Feedback always has the latest one.
pub const REPAIR_PROGRESS_PUBLISH_INTERVAL: Duration = Duration::from_millis(500);

/// Decision from a library command or observed fact (ticks, IO, and rescan timers only).
pub type LibraryOutcome =
    Outcome<core::convert::Infallible, LibraryTick, LibraryIoRequest, LibraryTimerKey>;

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
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{0}")]
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
    /// What the recording holds, from its MCAP summary; `None` without one or when it could not be read.
    pub contents: Option<RecordingContents>,
}

/// What a recording holds, from its MCAP summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingContents {
    /// Time from the first message to the last.
    pub duration: Duration,
    /// Topics of the video channels, sorted.
    pub video_topics: Vec<String>,
    /// How many other topics the recording has, such as telemetry.
    pub other_topic_count: u32,
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

impl LibraryRejection {
    /// Builds a rejection reason the Domain can return to clients.
    pub const fn new(reason: &'static str) -> Self {
        Self(reason)
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
