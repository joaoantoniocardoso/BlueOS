//! Active recording, the armed flag, bytes written and the record gate the data plane follows.

#![no_std]
#![expect(
    clippy::pub_use,
    reason = "the capture Block lives in block.rs; the crate root keeps public types and re-exports Capture"
)]

extern crate alloc;

mod block;

use core::{error::Error, fmt};

pub use block::Capture;

/// Backbone prefix for decoded MAVLink samples.
pub const MAVLINK_TOPIC_PREFIX: &str = "mavlink/";
/// Backbone prefix for raw MAVLink frames (ingress and egress).
pub const MAVLINK_RAW_TOPIC_PREFIX: &str = "mavlink_raw/";
/// Backbone prefix for camera manager video samples.
pub const VIDEO_TOPIC_PREFIX: &str = "video/";

/// Commands a client sends to the capture Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureRequest {
    /// Ask the data plane to open a file, or rotate when `rotate_if_active` and already active.
    StartRecording {
        /// When already recording, bump [`RecordGate::desired_file_generation`] so the Task rotates.
        rotate_if_active: bool,
    },
    /// Ask the data plane to finish the current file and stop recording.
    StopRecording,
    /// Changes capture settings; may start recording when [`CaptureSettings::auto_start_recording`] is enabled.
    UpdateSettings(CaptureSettings),
    /// Marks a video topic as recording for [`RecordGate::recording_video_topics`].
    StartVideoRecording {
        /// Zenoh topic prefix `video/...`.
        topic: alloc::string::String,
    },
    /// Removes a video topic from [`RecordGate::recording_video_topics`].
    StopVideoRecording {
        /// Video topic to stop.
        topic: alloc::string::String,
    },
}

/// Control-plane recording lifecycle; illegal combinations are unrepresentable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordingState {
    /// The user is not recording and the Task should close any open file.
    Idle,
    /// Recording was requested; waiting for the Task to report an open file.
    AwaitingMcapFile {
        /// Matches [`RecordGate::desired_file_generation`].
        file_generation: u64,
    },
    /// The Task reported an open file for this recording.
    Active(ActiveRecording),
}

/// Facts the data plane Task reports (full current value, handled idempotently).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureObservedFact {
    /// The Task opened the MCAP file for this generation.
    McapFileOpened {
        /// Matches [`RecordGate::desired_file_generation`] when first opened, or the rotation target.
        file_generation: u64,
        /// Base name of the opened file.
        file_name: alloc::string::String,
    },
    /// The Task finished writing this file generation.
    McapFileFinished {
        /// Generation that was closed.
        file_generation: u64,
    },
    /// Latest byte count for this file generation.
    RecordingBytesWritten {
        /// Generation whose file was written.
        file_generation: u64,
        /// Total bytes in the file so far.
        bytes: u64,
    },
    /// Latest count of samples left out of this file generation because they arrived faster than the disk took
    /// them.
    RecordingSamplesDropped {
        /// Generation whose file was written.
        file_generation: u64,
        /// Total samples dropped from the file so far.
        samples: u64,
    },
    /// Full current armed state (re-sent periodically so a dropped fact heals).
    ArmedChanged(bool),
}

/// Domain events emitted by the capture Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureEvent {
    /// The active file was replaced during rotation.
    RecordingRotated {
        /// Name of the new active file.
        file_name: alloc::string::String,
    },
    /// Recording was stopped.
    RecordingStopped,
}

/// Why a capture Command was rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureRejection {
    /// [`CaptureRequest::StopRecording`] arrived while [`RecordingState::Idle`].
    NotRecording,
}

/// Per-video-stream state used for the record gate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoStream {
    /// Same string as the map key.
    pub topic: alloc::string::String,
    /// Whether this stream is included in [`RecordGate::recording_video_topics`].
    pub is_recording: bool,
}

/// Projection the data plane Task reconciles against (D-27). Pure function of the Snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordGate {
    /// Whether the control plane wants the Task to keep an MCAP file open.
    pub recording_requested: bool,
    /// Latest armed fact from the vehicle.
    pub armed: bool,
    /// Copy of [`CaptureSettings::record_mavlink_only_when_armed`].
    pub record_mavlink_only_when_armed: bool,
    /// File generation the Task should open or rotate to; bumps on each rotation request.
    pub desired_file_generation: u64,
    /// Video topics that are actively being captured.
    pub recording_video_topics: alloc::collections::BTreeSet<alloc::string::String>,
}

/// The recording the Domain tracks after the data plane reports an open [`McapFile`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveRecording {
    /// Monotonic file identity the Task assigns; stale facts for other values are ignored.
    pub file_generation: u64,
    /// Base name of the file being written.
    pub file_name: alloc::string::String,
    /// Bytes reported by the data plane for this file.
    pub bytes_written: u64,
    /// Samples the data plane reported it left out of this file.
    pub samples_dropped: u64,
}

/// User settings that affect whether samples are written and when recording starts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureSettings {
    /// When set, MAVLink topics are recorded only while the vehicle is armed.
    pub record_mavlink_only_when_armed: bool,
    /// When set, recording starts as soon as settings allow; applied live on update.
    pub auto_start_recording: bool,
}

pub(crate) type CaptureOutcome = blueos_domain::Outcome<
    CaptureEvent,
    core::convert::Infallible,
    core::convert::Infallible,
    core::convert::Infallible,
>;

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            record_mavlink_only_when_armed: true,
            auto_start_recording: true,
        }
    }
}

impl fmt::Display for CaptureRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRecording => formatter.write_str("not recording"),
        }
    }
}

impl Error for CaptureRejection {}
