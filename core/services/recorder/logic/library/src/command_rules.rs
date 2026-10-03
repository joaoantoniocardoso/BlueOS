//! Which library Commands are allowed for one recording row.

use alloc::{string::String, vec::Vec};
use core::time::Duration;

use blueos_domain::Now;
use blueos_jobs::JobControl;

/// Minimum age before an unindexed file may be repaired (avoids racing an active writer).
pub const RECENTLY_WRITTEN_DELAY: Duration = Duration::from_secs(10);

/// Endpoint name for [`DeleteRecording`](crate::LibraryRequest::DeleteRecording).
pub const DELETE_RECORDING: &str = "DeleteRecording";
/// Endpoint name for the repair Command.
pub const REPAIR_RECORDING: &str = "RepairRecording";
/// Endpoint name of the Kernel's control that cancels a repair's Job.
pub const CANCEL_JOB: &str = JobControl::Cancel.endpoint_name();
/// Endpoint name for the snapshot Command.
pub const SNAPSHOT_RECORDING: &str = "SnapshotRecording";

/// Inputs shared by rejection checks and [`allowed_operations`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordingCommandContext<'a> {
    /// Relative path of this library row.
    pub relative_path: &'a str,
    /// Relative path of the file currently being written (today the same as the active base name).
    pub active_recording_relative_path: Option<&'a str>,
    /// Whether the scan found this path.
    pub in_library: bool,
    /// Whether delete is in progress for this path.
    pub deleting: bool,
    /// Whether repair is in progress for this path.
    pub repairing: bool,
    /// Whether a snapshot is in progress for this path.
    pub snapshotting: bool,
    /// Whether the MCAP summary is present.
    pub indexed: bool,
    /// Modification time from the last scan (Unix seconds).
    pub modified_unix_seconds: i64,
    /// Injected time for the Command.
    pub now: Now,
}

/// Command endpoint names the library accepts for this recording.
pub fn allowed_operations(context: &RecordingCommandContext<'_>) -> Vec<String> {
    let mut operations = Vec::new();
    if repair_recording_rejection(context).is_none() {
        operations.push(REPAIR_RECORDING.into());
    }
    if cancel_repair_rejection(context).is_none() {
        operations.push(CANCEL_JOB.into());
    }
    if snapshot_recording_rejection(context).is_none() {
        operations.push(SNAPSHOT_RECORDING.into());
    }
    if delete_recording_rejection(context).is_none() {
        operations.push(DELETE_RECORDING.into());
    }
    operations
}

/// Why delete is rejected, or `None` when it would apply.
pub fn delete_recording_rejection(context: &RecordingCommandContext<'_>) -> Option<&'static str> {
    if !context.in_library {
        return Some("Recording not found.");
    }
    if context.deleting || context.repairing || context.snapshotting {
        return Some("This recording is being processed.");
    }
    if context.active_recording_relative_path == Some(context.relative_path) {
        return Some("This recording is still being written.");
    }
    None
}

/// Why repair is rejected, or `None` when it would apply.
pub fn repair_recording_rejection(context: &RecordingCommandContext<'_>) -> Option<&'static str> {
    if !context.in_library {
        return Some("Recording not found.");
    }
    if context.repairing {
        return Some("This recording is already being repaired.");
    }
    if context.snapshotting {
        return Some("This recording is being processed.");
    }
    if context.indexed {
        return Some("This recording already has an index.");
    }
    if context.active_recording_relative_path == Some(context.relative_path) {
        return Some("This recording is still being written. Try again once it is finished.");
    }
    if recently_written(context.modified_unix_seconds, context.now) {
        return Some("This recording is still being written. Try again once it is finished.");
    }
    None
}

/// Why cancel repair is rejected, or `None` when it would apply.
pub fn cancel_repair_rejection(context: &RecordingCommandContext<'_>) -> Option<&'static str> {
    if !context.repairing {
        return Some("This recording is not being repaired.");
    }
    None
}

/// Why snapshot is rejected, or `None` when it would apply.
pub fn snapshot_recording_rejection(context: &RecordingCommandContext<'_>) -> Option<&'static str> {
    if !context.in_library {
        return Some("Recording not found.");
    }
    if context.snapshotting || context.repairing {
        return Some("This recording is being processed.");
    }
    None
}

fn recently_written(modified_unix_seconds: i64, now: Now) -> bool {
    let modified = Duration::from_secs(modified_unix_seconds.max(0) as u64);
    now.wall.saturating_sub(modified) < RECENTLY_WRITTEN_DELAY
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: Now = Now {
        wall: Duration::from_secs(20_000),
        monotonic: Duration::ZERO,
    };

    fn context(relative_path: &str, in_library: bool) -> RecordingCommandContext<'_> {
        RecordingCommandContext {
            relative_path,
            active_recording_relative_path: None,
            in_library,
            deleting: false,
            repairing: false,
            snapshotting: false,
            indexed: false,
            modified_unix_seconds: 1_000,
            now: NOW,
        }
    }

    #[test]
    fn snapshot_rejects_missing_file() {
        let context = context("missing.mcap", false);
        assert_eq!(
            snapshot_recording_rejection(&context),
            Some("Recording not found.")
        );
    }

    #[test]
    fn snapshot_rejects_while_repair_runs() {
        let mut context = context("live.mcap", true);
        context.repairing = true;
        assert_eq!(
            snapshot_recording_rejection(&context),
            Some("This recording is being processed.")
        );
    }

    #[test]
    fn snapshot_allowed_for_active_recording_row() {
        let mut context = context("live.mcap", true);
        context.active_recording_relative_path = Some("live.mcap");
        assert!(snapshot_recording_rejection(&context).is_none());
    }

    #[test]
    fn delete_rejected_while_snapshot_runs() {
        let mut context = context("live.mcap", true);
        context.snapshotting = true;
        assert_eq!(
            delete_recording_rejection(&context),
            Some("This recording is being processed.")
        );
    }
}
