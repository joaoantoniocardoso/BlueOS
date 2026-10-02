//! Conversions between the Recorder Domain and its public Messages, plus [`RecorderSettings`].

#![no_std]

extern crate alloc;

pub mod endpoints;

use alloc::{string::String, vec::Vec};
use core::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use blueos_idl::msg::blueos_recorder_msgs::{
    RecordingPolicy, RecordingState, StartRecordingCommand, StopRecordingCommand,
};
use blueos_recorder_capture::{CaptureSettings, RecordingState as DomainRecordingState};
use blueos_recorder_domain::{RecorderDomain, RecorderRequest, RecorderSnapshot};
use blueos_settings::SettingsSchema;

use crate::endpoints::Conversions;

/// Python-compatible persisted settings for the Recorder (owned by the Kernel).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecorderSettings {
    /// Settings document version.
    #[serde(rename = "VERSION")]
    pub version: NonZeroU32,
    /// When set, MAVLink topics are recorded only while the vehicle is armed.
    pub record_mavlink_only_when_armed: bool,
    /// When set, recording starts as soon as settings allow.
    pub auto_start_recording: bool,
}

impl Default for RecorderSettings {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            record_mavlink_only_when_armed: true,
            auto_start_recording: true,
        }
    }
}

impl SettingsSchema for RecorderSettings {
    const VERSION: NonZeroU32 = NonZeroU32::new(1).unwrap();

    fn migrate(_data: &mut serde_json::Value) -> Result<(), blueos_settings::SettingsError> {
        Ok(())
    }

    fn restart_required_fields() -> &'static [&'static str] {
        &[]
    }
}

impl RecorderSettings {
    /// Maps into the capture Block settings.
    pub fn into_capture_settings(self) -> CaptureSettings {
        CaptureSettings {
            record_mavlink_only_when_armed: self.record_mavlink_only_when_armed,
            auto_start_recording: self.auto_start_recording,
        }
    }

    /// Builds from the capture Block settings and document version.
    pub fn from_capture(settings: &CaptureSettings) -> Self {
        Self {
            version: Self::VERSION,
            record_mavlink_only_when_armed: settings.record_mavlink_only_when_armed,
            auto_start_recording: settings.auto_start_recording,
        }
    }

    /// Maps from the wire [`RecordingPolicy`].
    pub fn from_recording_policy(policy: &RecordingPolicy) -> Self {
        Self {
            version: Self::VERSION,
            record_mavlink_only_when_armed: policy.record_mavlink_only_when_armed,
            auto_start_recording: policy.auto_start_recording,
        }
    }
}

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
