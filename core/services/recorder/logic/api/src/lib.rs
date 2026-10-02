//! Conversions between the Recorder Domain and its public Messages.

#![no_std]

extern crate alloc;

pub mod endpoints;

use alloc::{string::String, vec::Vec};

use blueos_idl::msg::blueos_recorder_msgs::{
    RecordingFile, RecordingFileState as WireRecordingFileState, RecordingLibrary, RecordingState,
    StartRecordingCommand, StopRecordingCommand,
};
use blueos_recorder_capture::RecordingState as DomainRecordingState;
use blueos_recorder_domain::{RecorderDomain, RecorderRequest, RecorderSnapshot};
use blueos_recorder_library::RecordingFileState;

use crate::endpoints::Conversions;

impl Conversions for RecorderDomain {
    fn start(request: StartRecordingCommand) -> RecorderRequest {
        RecorderRequest::StartRecording {
            rotate_if_active: request.rotate_if_active,
        }
    }

    fn stop(_request: StopRecordingCommand) -> RecorderRequest {
        RecorderRequest::StopRecording
    }

    fn recording(snapshot: &RecorderSnapshot) -> RecordingState {
        recorder_session_state(snapshot)
    }

    fn library(snapshot: &RecorderSnapshot) -> RecordingLibrary {
        recording_library(snapshot)
    }
}

/// Projects the published `recording` State from the Snapshot (wire names mapped here).
///
/// Wire field names use legacy "session" wording (see GLOSSARY).
pub fn recorder_session_state(snapshot: &RecorderSnapshot) -> RecordingState {
    let gate = snapshot.capture.record_gate();
    let (session_active, current_file, session_bytes_written) = match &snapshot.capture.recording {
        DomainRecordingState::Idle => (false, String::new(), 0),
        DomainRecordingState::AwaitingMcapFile { .. } => (true, String::new(), 0),
        DomainRecordingState::Active(active) => {
            (true, active.file_name.clone(), active.bytes_written)
        }
    };
    RecordingState {
        armed: snapshot.capture.armed,
        session_active,
        current_file,
        session_bytes_written,
        recording_video_topics: gate
            .recording_video_topics
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
    }
}

/// Projects the published `library` State from the Snapshot.
pub fn recording_library(snapshot: &RecorderSnapshot) -> RecordingLibrary {
    RecordingLibrary {
        files: snapshot
            .library
            .entries()
            .iter()
            .map(recording_file_message)
            .collect(),
    }
}

fn recording_file_message(entry: &blueos_recorder_library::RecordingFileEntry) -> RecordingFile {
    let (sec, nanosec) = unix_seconds_to_time(entry.created_unix_seconds);
    RecordingFile {
        path: entry.path.clone(),
        name: entry.name.clone(),
        size_bytes: entry.size_bytes,
        created: blueos_idl::msg::builtin_interfaces::Time { sec, nanosec },
        state: recording_file_state_wire(entry.state),
        repair_bytes_processed: 0,
        repair_total_bytes: 0,
        repair_bytes_per_second: 0.0,
        repair_error: String::new(),
        allowed_operations: entry.allowed_operations.clone(),
    }
}

fn recording_file_state_wire(state: RecordingFileState) -> WireRecordingFileState {
    match state {
        RecordingFileState::Recording => WireRecordingFileState::Recording,
        RecordingFileState::Ready => WireRecordingFileState::Ready,
        RecordingFileState::NeedsRepair => WireRecordingFileState::NeedsRepair,
        RecordingFileState::Repairing => WireRecordingFileState::Repairing,
    }
}

fn unix_seconds_to_time(seconds: i64) -> (i32, u32) {
    (i32::try_from(seconds).unwrap_or(i32::MAX), 0)
}
