//! MAVLink camera protocol types shared by the Block and its tests.

use alloc::string::String;
use core::time::Duration;

use blueos_domain::{Now, Outcome};

/// Topic the camera manager publishes raw MAVLink on.
pub const RAW_MAVLINK_OUT_TOPIC: &str = "mavlink_raw/out";
/// Topic the recorder publishes MAVLink replies on.
pub const RAW_MAVLINK_IN_TOPIC: &str = "mavlink_raw/in";

/// Minimum spacing between periodic capture status replies.
pub(crate) const MIN_CAPTURE_STATUS_INTERVAL: Duration = Duration::from_millis(100);
/// Maximum spacing when the camera manager requests a very low rate.
pub(crate) const MAX_CAPTURE_STATUS_INTERVAL: Duration = Duration::from_secs(1);
/// Default spacing when the command omits a usable rate.
pub(crate) const DEFAULT_CAPTURE_STATUS_INTERVAL: Duration = Duration::from_secs(1);

/// MAVLink `video_status` when the stream is not recording.
pub const VIDEO_CAPTURE_STATUS_IDLE: u8 = 0;
/// MAVLink `video_status` when the stream is recording.
pub const VIDEO_CAPTURE_STATUS_RECORDING: u8 = 1;

/// Facts the MAVLink ingress Task reports from raw frames.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CamerasObservedFact {
    /// A camera component sent its first heartbeat.
    CameraHeartbeat {
        /// Camera ids.
        camera: SystemAndComponent,
    },
    /// A camera stream was registered from `VIDEO_STREAM_INFORMATION`.
    RegisterVideoStream {
        /// `video/...` topic written to the backbone.
        topic: String,
        /// Source camera ids.
        camera: SystemAndComponent,
    },
    /// Whether the camera supports video capture.
    SetCameraRecordingCapability {
        /// Camera ids.
        camera: SystemAndComponent,
        /// When false, start capture is rejected.
        capture_video: bool,
    },
    /// A `COMMAND_LONG` capture command from the camera manager.
    CameraCaptureCommand {
        /// Which capture command was sent.
        command: CaptureCommandKind,
        /// System and component that sent the command, which the ack is addressed to.
        sender: SystemAndComponent,
        /// MAVLink target system.
        target_system: u8,
        /// MAVLink target component.
        target_component: u8,
        /// Minimum spacing between periodic capture status replies.
        status_interval: Duration,
    },
}

/// Timer keys; one capture status timer per video topic.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CamerasTimerKey {
    /// Periodic capture status for one stream.
    CaptureStatus {
        /// `video/...` topic.
        topic: String,
    },
}

/// Ticks scheduled by this Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CamerasTick {
    /// Send [`CamerasIoRequest::CaptureStatus`] and re-arm this stream's timer.
    CaptureStatus {
        /// `video/...` topic.
        topic: String,
    },
}

/// IO this Block performs through the MAVLink egress topic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CamerasIoRequest {
    /// Publish a `COMMAND_ACK` frame.
    CommandAck {
        /// Camera that sends the ack.
        camera: SystemAndComponent,
        /// Sender of the command, which the ack is addressed to.
        recipient: SystemAndComponent,
        /// MAVLink command being acknowledged.
        command: CaptureCommandKind,
        /// Whether the command was accepted.
        accepted: bool,
    },
    /// Publish a `CAMERA_CAPTURE_STATUS` frame.
    CaptureStatus {
        /// Camera that sends the status.
        camera: SystemAndComponent,
        /// MAVLink `video_status` field.
        video_status: u8,
        /// Elapsed recording time in milliseconds.
        recording_time_ms: u32,
    },
    /// Publish a `COMMAND_LONG` discovery request to a camera.
    RequestDiscovery {
        /// Camera that should receive the request.
        camera: SystemAndComponent,
        /// Which information message to request.
        message: DiscoveryMessageKind,
    },
}

/// Result of cameras IO executed by the Kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CamerasIoResult {
    /// Publishing a MAVLink reply to the backbone failed.
    PublishFailed,
}

/// Which capture command the camera manager sent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureCommandKind {
    /// Start video capture for the addressed camera stream.
    StartCapture,
    /// Stop video capture for the addressed camera stream.
    StopCapture,
    /// One-shot capture status reply.
    RequestCaptureStatus,
}

/// MAVLink discovery messages sent after a camera heartbeat.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryMessageKind {
    /// Request `CAMERA_INFORMATION`.
    CameraInformation,
    /// Request `VIDEO_STREAM_INFORMATION`.
    VideoStreamInformation,
}

/// Per-stream state for capture status timers and recording time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoStream {
    /// Camera that owns this stream.
    pub camera: SystemAndComponent,
    /// Whether MAVLink capture is active on this stream.
    pub is_recording: bool,
    /// Minimum spacing between periodic capture status replies.
    pub status_interval: Duration,
    /// Monotonic time when recording started, for elapsed capture status reporting.
    pub recording_started_at: Option<Duration>,
}

/// A MAVLink system and component id pair.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SystemAndComponent {
    /// MAVLink system id.
    pub system_id: u8,
    /// MAVLink component id.
    pub component_id: u8,
}

pub(crate) type CamerasOutcome =
    Outcome<core::convert::Infallible, CamerasTick, CamerasIoRequest, CamerasTimerKey>;

/// Inputs for [`Cameras::handle_capture_command`].
pub(crate) struct CaptureCommandInput {
    pub command: CaptureCommandKind,
    pub sender: SystemAndComponent,
    pub target_system: u8,
    pub target_component: u8,
    pub status_interval: Duration,
}

/// Timing and command metadata when arming capture on one stream.
pub(crate) struct StartCaptureDispatch<'input> {
    pub input: &'input CaptureCommandInput,
    pub interval: Duration,
    pub now: Now,
    pub accepted: bool,
}
