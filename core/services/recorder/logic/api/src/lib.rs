//! Conversions between the Recorder Domain and its public Messages.

#![no_std]

extern crate alloc;

pub mod endpoints;

use alloc::{string::String, vec::Vec};

use blueos_idl::msg::blueos_recorder_msgs::{
    RecordingState, StartRecordingCommand, StopRecordingCommand,
};
use blueos_recorder_capture::RecordingState as DomainRecordingState;
use blueos_recorder_domain::{RecorderDomain, RecorderRequest, RecorderSnapshot};

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
