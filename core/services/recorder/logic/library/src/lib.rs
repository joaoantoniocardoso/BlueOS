#![no_std]

extern crate alloc;

mod path;
mod timestamp;

#[cfg(test)]
mod tests;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;

use blueos_cqrs::{Decision, Domain, Effect, TimerId};
use blueos_jobs::{JobId, Jobs};

pub use path::{PathValidationError, validate_relative_recording_path};
pub use timestamp::created_unix_seconds_from_filename;

/// Minimum age before an unindexed file may be repaired (avoids racing an active writer).
pub const RECENTLY_WRITTEN_SECONDS: i64 = 10;
// ponytail: timer rescan; switch to inotify if external writers or large folders make it costly.
/// How often the library rescans the recordings folder while idle.
pub const RESCAN_INTERVAL: Duration = Duration::from_secs(5);
/// Timer id for periodic library rescans.
pub const RESCAN_TIMER: TimerId = TimerId(200);

/// Lifecycle state of one recording in the published library view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordingState {
    Recording,
    Ready,
    NeedsRepair,
    Repairing,
}

/// One row in the recording library state published to clients.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordingEntry {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub created_unix_seconds: i64,
    pub state: RecordingState,
    pub repair_bytes_processed: u64,
    pub repair_total_bytes: u64,
    pub repair_bytes_per_second: f64,
    pub repair_error: String,
}

/// Metadata collected from disk during a library scan.
#[derive(Clone, Debug, PartialEq)]
pub struct ScannedRecording {
    pub relative_path: String,
    pub name: String,
    pub size_bytes: u64,
    pub modified_unix_seconds: i64,
    pub indexed: bool,
}

/// Long-running library work started by a command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationKind {
    Repair,
    Snapshot,
    Delete,
}

/// Outcome of repair, snapshot, or delete work emitted to subscribers.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordingOperationEvent {
    pub operation: OperationKind,
    pub path: String,
    pub output_path: String,
    pub succeeded: bool,
    pub cancelled: bool,
    pub error: String,
}

/// In-memory library domain state: scan cache plus in-flight operations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LibrarySnapshot {
    pub entries: Vec<RecordingEntry>,
    scanned: BTreeMap<String, ScannedRecording>,
    repairing: BTreeMap<String, RepairProgress>,
    snapshotting: BTreeMap<String, String>,
    deleting: BTreeMap<String, ()>,
}

#[derive(Clone, Debug, PartialEq)]
struct RepairProgress {
    bytes_processed: u64,
    total_bytes: u64,
    started_unix_seconds: i64,
    bytes_per_second: f64,
}

/// Commands handled by the recording library domain.
#[derive(Clone, Debug, PartialEq)]
pub enum LibraryCommand {
    RepairRecording {
        path: String,
        now_unix_seconds: i64,
    },
    CancelRepair {
        path: String,
    },
    DeleteRecording {
        path: String,
    },
    SnapshotRecording {
        path: String,
        now_unix_seconds: i64,
    },
    RescanTick,
    ScanCompleted {
        recordings: Vec<ScannedRecording>,
        now_unix_seconds: i64,
    },
    RepairProgress {
        path: String,
        bytes_processed: u64,
        total_bytes: u64,
        now_unix_seconds: i64,
    },
    OperationFinished {
        operation: OperationKind,
        path: String,
        output_path: String,
        succeeded: bool,
        cancelled: bool,
        error: String,
    },
}

/// Events emitted when library work completes or fails.
#[derive(Clone, Debug, PartialEq)]
pub enum LibraryEvent {
    Operation(RecordingOperationEvent),
}

/// Blocking IO requested by the library domain (served outside the inbox).
#[derive(Clone, Debug, PartialEq)]
pub enum LibraryIo {
    Scan,
    Repair { path: String },
    CancelRepair { path: String },
    Delete { path: String },
    Snapshot { path: String, output_path: String },
}

/// Library has no synchronous queries.
pub enum LibraryQuery {}

/// Placeholder view type for unused library queries.
pub struct LibraryQueryView;

#[derive(Clone)]
pub enum LibraryJobSpec {}

/// CQRS domain for the recording library.
pub struct LibraryDomain;

impl Domain for LibraryDomain {
    type Command = LibraryCommand;
    type Event = LibraryEvent;
    type Query = LibraryQuery;
    type View = LibraryQueryView;
    type Snapshot = LibrarySnapshot;
    type IoRequest = LibraryIo;
    type JobSpec = LibraryJobSpec;

    fn handle_command(
        snapshot: &mut Self::Snapshot,
        _jobs: &mut Jobs<Self::JobSpec>,
        command: Self::Command,
    ) -> Decision<Self> {
        handle_library_command(snapshot, command, None)
    }

    fn handle_query(
        _snapshot: &Self::Snapshot,
        _jobs: &Jobs<Self::JobSpec>,
        _query: Self::Query,
    ) -> Self::View {
        LibraryQueryView
    }

    fn io_from_job(_job_id: JobId, _job_spec: &Self::JobSpec) -> Self::IoRequest {
        LibraryIo::Scan
    }
}

/// Applies one library command and returns effects, events, or a rejection reason.
pub fn handle_library_command(
    snapshot: &mut LibrarySnapshot,
    command: LibraryCommand,
    active_session_relative_path: Option<&str>,
) -> Decision<LibraryDomain> {
    match command {
        LibraryCommand::RescanTick => Decision {
            events: Vec::new(),
            effects: vec![
                Effect::Io(LibraryIo::Scan),
                Effect::Schedule {
                    after: RESCAN_INTERVAL,
                    timer: RESCAN_TIMER,
                    command: LibraryCommand::RescanTick,
                },
            ],
            rejection: None,
        },
        LibraryCommand::ScanCompleted {
            recordings,
            now_unix_seconds: _,
        } => {
            snapshot.scanned.clear();
            for recording in recordings {
                snapshot
                    .scanned
                    .insert(recording.relative_path.clone(), recording);
            }
            rebuild_entries(snapshot, active_session_relative_path);
            Decision::new()
        }
        LibraryCommand::RepairRecording {
            path,
            now_unix_seconds,
        } => {
            if let Err(reason) = validate_relative_recording_path(&path) {
                return Decision::reject(reason);
            }
            if snapshot.repairing.contains_key(&path) {
                return Decision::reject("This recording is already being repaired.");
            }
            let scanned = snapshot.scanned.get(&path);
            if scanned.is_none() {
                return Decision::reject("Recording not found.");
            }
            let scanned = scanned.expect("checked");
            if scanned.indexed {
                return Decision::reject("This recording already has an index.");
            }
            if is_active_recording(active_session_relative_path, &path) {
                return Decision::reject(
                    "This recording is still being written. Try again once it is finished.",
                );
            }
            let written_seconds_ago = now_unix_seconds - scanned.modified_unix_seconds;
            if written_seconds_ago < RECENTLY_WRITTEN_SECONDS {
                return Decision::reject(
                    "This recording is still being written. Try again once it is finished.",
                );
            }
            let total_bytes = scanned.size_bytes;
            snapshot.repairing.insert(
                path.clone(),
                RepairProgress {
                    bytes_processed: 0,
                    total_bytes,
                    started_unix_seconds: now_unix_seconds,
                    bytes_per_second: 0.0,
                },
            );
            rebuild_entries(snapshot, active_session_relative_path);
            Decision {
                events: Vec::new(),
                effects: vec![Effect::Io(LibraryIo::Repair { path })],
                rejection: None,
            }
        }
        LibraryCommand::CancelRepair { path } => {
            if let Err(reason) = validate_relative_recording_path(&path) {
                return Decision::reject(reason);
            }
            if !snapshot.repairing.contains_key(&path) {
                return Decision::reject("This recording is not being repaired.");
            }
            Decision {
                events: Vec::new(),
                effects: vec![Effect::Io(LibraryIo::CancelRepair { path })],
                rejection: None,
            }
        }
        LibraryCommand::DeleteRecording { path } => {
            if let Err(reason) = validate_relative_recording_path(&path) {
                return Decision::reject(reason);
            }
            if !snapshot.scanned.contains_key(&path) {
                return Decision::reject("Recording not found.");
            }
            if snapshot.repairing.contains_key(&path)
                || snapshot.snapshotting.contains_key(&path)
                || snapshot.deleting.contains_key(&path)
            {
                return Decision::reject("This recording is being processed.");
            }
            if is_active_recording(active_session_relative_path, &path) {
                return Decision::reject("This recording is still being written.");
            }
            snapshot.deleting.insert(path.clone(), ());
            rebuild_entries(snapshot, active_session_relative_path);
            Decision {
                events: Vec::new(),
                effects: vec![Effect::Io(LibraryIo::Delete { path })],
                rejection: None,
            }
        }
        LibraryCommand::SnapshotRecording {
            path,
            now_unix_seconds,
        } => {
            if let Err(reason) = validate_relative_recording_path(&path) {
                return Decision::reject(reason);
            }
            if !snapshot.scanned.contains_key(&path) {
                return Decision::reject("Recording not found.");
            }
            if snapshot.snapshotting.contains_key(&path)
                || snapshot.repairing.contains_key(&path)
                || snapshot.deleting.contains_key(&path)
            {
                return Decision::reject("This recording is being processed.");
            }
            let output_path = snapshot_output_path(&path, now_unix_seconds);
            snapshot
                .snapshotting
                .insert(path.clone(), output_path.clone());
            rebuild_entries(snapshot, active_session_relative_path);
            Decision {
                events: Vec::new(),
                effects: vec![Effect::Io(LibraryIo::Snapshot { path, output_path })],
                rejection: None,
            }
        }
        LibraryCommand::RepairProgress {
            path,
            bytes_processed,
            total_bytes,
            now_unix_seconds,
        } => {
            if let Some(progress) = snapshot.repairing.get_mut(&path) {
                progress.bytes_processed = bytes_processed.min(total_bytes);
                progress.total_bytes = total_bytes;
                let elapsed = now_unix_seconds
                    .saturating_sub(progress.started_unix_seconds)
                    .max(1) as f64;
                progress.bytes_per_second = progress.bytes_processed as f64 / elapsed;
            }
            rebuild_entries(snapshot, active_session_relative_path);
            Decision::new()
        }
        LibraryCommand::OperationFinished {
            operation,
            path,
            output_path,
            succeeded,
            cancelled,
            error,
        } => {
            snapshot.repairing.remove(&path);
            snapshot.snapshotting.remove(&path);
            snapshot.deleting.remove(&path);
            if operation == OperationKind::Repair && cancelled {
                rebuild_entries(snapshot, active_session_relative_path);
                let event = RecordingOperationEvent {
                    operation,
                    path,
                    output_path,
                    succeeded: false,
                    cancelled: true,
                    error: String::new(),
                };
                return operation_finished_decision(event);
            }
            if operation == OperationKind::Repair && !succeeded && !cancelled {
                if let Some(entry) = snapshot.entries.iter_mut().find(|entry| entry.path == path) {
                    entry.repair_error = error.clone();
                }
            } else if operation == OperationKind::Repair
                && succeeded
                && let Some(entry) = snapshot.entries.iter_mut().find(|entry| entry.path == path)
            {
                entry.repair_error.clear();
            }
            rebuild_entries(snapshot, active_session_relative_path);
            let event = RecordingOperationEvent {
                operation,
                path,
                output_path,
                succeeded,
                cancelled,
                error: if cancelled { String::new() } else { error },
            };
            operation_finished_decision(event)
        }
    }
}

fn operation_finished_decision(event: RecordingOperationEvent) -> Decision<LibraryDomain> {
    Decision {
        events: vec![LibraryEvent::Operation(event)],
        effects: vec![Effect::Io(LibraryIo::Scan)],
        rejection: None,
    }
}

/// Schedules the next periodic rescan after the standard interval.
pub fn arm_rescan_schedule() -> Effect<LibraryCommand, LibraryIo> {
    Effect::Schedule {
        after: RESCAN_INTERVAL,
        timer: RESCAN_TIMER,
        command: LibraryCommand::RescanTick,
    }
}

/// Maps scan and in-flight repair state to the state shown in the library.
pub fn derive_recording_state(
    relative_path: &str,
    active_session_relative_path: Option<&str>,
    repairing: bool,
    indexed: bool,
) -> RecordingState {
    if repairing {
        return RecordingState::Repairing;
    }
    if is_active_recording(active_session_relative_path, relative_path) {
        return RecordingState::Recording;
    }
    if indexed {
        return RecordingState::Ready;
    }
    RecordingState::NeedsRepair
}

/// Relative path for a snapshot copy written next to a recording still being captured.
pub fn snapshot_output_path(source_path: &str, now_unix_seconds: i64) -> String {
    let source = source_path.rsplit('/').next().unwrap_or(source_path);
    let stem = source
        .strip_suffix(".mcap")
        .or_else(|| source.strip_suffix(".MCAP"))
        .unwrap_or(source);
    let timestamp = format_snapshot_timestamp(now_unix_seconds);
    let parent = source_path.rsplit_once('/').map(|(parent, _)| parent);
    let file_name = format!("{stem}.snapshot-{timestamp}Z.mcap");
    match parent {
        Some(prefix) if !prefix.is_empty() => format!("{prefix}/{file_name}"),
        _ => file_name,
    }
}

fn format_snapshot_timestamp(unix_seconds: i64) -> String {
    let second = unix_seconds.rem_euclid(60);
    let minute = (unix_seconds / 60).rem_euclid(60);
    let hour = (unix_seconds / 3_600).rem_euclid(24);
    let days = unix_seconds.div_euclid(86_400);
    let (year, month, day) = epoch_days_to_civil(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}-{minute:02}-{second:02}")
}

fn epoch_days_to_civil(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 {
        days / 146_097
    } else {
        (days - 146_096) / 146_097
    };
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month + 2) / 5 + 1;
    let month = month + if month < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

fn is_active_recording(active_session_relative_path: Option<&str>, path: &str) -> bool {
    active_session_relative_path == Some(path)
}

fn rebuild_entries(snapshot: &mut LibrarySnapshot, active_session_relative_path: Option<&str>) {
    let mut entries = Vec::new();
    for (path, scanned) in &snapshot.scanned {
        let repairing = snapshot.repairing.get(path);
        let state = derive_recording_state(
            path,
            active_session_relative_path,
            repairing.is_some(),
            scanned.indexed,
        );
        let (repair_bytes_processed, repair_total_bytes, repair_bytes_per_second) =
            if let Some(progress) = repairing {
                (
                    progress.bytes_processed,
                    progress.total_bytes,
                    progress.bytes_per_second,
                )
            } else {
                (0, 0, 0.0)
            };
        let repair_error = snapshot
            .entries
            .iter()
            .find(|entry| entry.path == *path)
            .map(|entry| entry.repair_error.clone())
            .unwrap_or_default();
        let created_unix_seconds =
            created_unix_seconds_from_filename(&scanned.name, scanned.modified_unix_seconds);
        entries.push(RecordingEntry {
            path: path.clone(),
            name: scanned.name.clone(),
            size_bytes: scanned.size_bytes,
            created_unix_seconds,
            state,
            repair_bytes_processed,
            repair_total_bytes,
            repair_bytes_per_second,
            repair_error,
        });
    }
    entries.sort_by_key(|entry| core::cmp::Reverse(entry.created_unix_seconds));
    snapshot.entries = entries;
}

/// Effects to run when the library is first wired into the recorder domain.
pub fn initial_rescan_effects() -> Vec<Effect<LibraryCommand, LibraryIo>> {
    vec![Effect::Io(LibraryIo::Scan), arm_rescan_schedule()]
}
