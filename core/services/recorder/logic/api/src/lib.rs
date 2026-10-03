//! Conversions between the Recorder Domain and its public Messages.

#![no_std]

extern crate alloc;

pub mod endpoints;

use alloc::{string::String, vec::Vec};

use blueos_idl::msg::blueos_recorder_msgs::{
    RecordingFile, RecordingFileState as WireRecordingFileState, RecordingLibrary,
    RecordingOperation, RecordingOperationOperation, RecordingState, StartRecordingCommand,
    StopRecordingCommand,
};
use blueos_recorder_capture::RecordingState as DomainRecordingState;
use blueos_recorder_domain::{RecorderDomain, RecorderEvent, RecorderRequest, RecorderSnapshot};
use blueos_recorder_library::{
    RecordingFileState, RecordingOperationEvent, RecordingOperationKind, RepairFailure,
};

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

    fn operation(event: &RecorderEvent) -> Option<RecordingOperation> {
        match event {
            RecorderEvent::RecordingOperation(operation) => {
                Some(recording_operation_message(operation))
            }
            RecorderEvent::Capture(_) => None,
        }
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
        repair_bytes_processed: entry.repair_bytes_processed,
        repair_total_bytes: entry.repair_total_bytes,
        repair_bytes_per_second: entry.repair_bytes_per_second,
        repair_error: entry.repair_error.clone(),
        allowed_operations: entry.allowed_operations.clone(),
        repair_job_id: entry.repair_job_id.map(String::from).unwrap_or_default(),
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

fn recording_operation_message(event: &RecordingOperationEvent) -> RecordingOperation {
    let operation = match event.operation {
        RecordingOperationKind::Repair => RecordingOperationOperation::Repair,
        RecordingOperationKind::Snapshot => RecordingOperationOperation::Snapshot,
    };
    RecordingOperation {
        operation,
        path: event.path.clone(),
        output_path: event.output_path.clone(),
        succeeded: event.succeeded,
        cancelled: event.cancelled,
        error: repair_failure_wire(&event.failure, event.cancelled),
    }
}

fn repair_failure_wire(failure: &RepairFailure, cancelled: bool) -> String {
    if cancelled {
        return String::new();
    }
    match failure {
        RepairFailure::None => String::new(),
        RepairFailure::Io => "Filesystem operation failed.".into(),
        RepairFailure::Rewrite => "MCAP rewrite failed.".into(),
        RepairFailure::Replace => "Could not replace the recording file.".into(),
        RepairFailure::Message(message) => message.clone(),
    }
}
