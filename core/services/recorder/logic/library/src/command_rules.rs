//! Which library Commands are allowed for one recording row.

use alloc::{string::String, vec, vec::Vec};

/// Endpoint name for [`DeleteRecording`](crate::LibraryRequest::DeleteRecording).
pub const DELETE_RECORDING: &str = "DeleteRecording";

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
}

/// Command endpoint names the library accepts for this recording (delete only).
pub fn allowed_operations(context: &RecordingCommandContext<'_>) -> Vec<String> {
    if delete_recording_rejection(context).is_none() {
        vec![DELETE_RECORDING.into()]
    } else {
        Vec::new()
    }
}

/// Why delete is rejected, or `None` when it would apply.
pub fn delete_recording_rejection(context: &RecordingCommandContext<'_>) -> Option<&'static str> {
    if !context.in_library {
        return Some("Recording not found.");
    }
    if context.deleting {
        return Some("This recording is being processed.");
    }
    if context.active_recording_relative_path == Some(context.relative_path) {
        return Some("This recording is still being written.");
    }
    None
}
