//! Raw MAVLink parse and serialize for the recorder (MCM external recorder contract).

#![expect(
    clippy::pub_use,
    reason = "adapter crate re-exports encode helpers at the root"
)]

mod camera;
mod encode;
mod vehicle;

use std::collections::BTreeSet;
use std::string::String;
use std::vec::Vec;

use mavlink::MessageData;
use mavlink::dialects::ardupilotmega::{
    CAMERA_INFORMATION_DATA, COMMAND_LONG_DATA, HEARTBEAT_DATA, MavCmd, MavComponent, MavMessage,
    MavType, VIDEO_STREAM_INFORMATION_DATA,
};
use mavlink_codec::PacketRef;
use tracing::{trace, warn};

pub use camera::video_topic_from_name;
pub use encode::{
    build_camera_capture_status, build_command_ack, build_discovery_request, encode_command_long,
    encode_mavlink_message, test_camera_capture_frame, test_vehicle_heartbeat_frame,
};
pub use vehicle::armed_from_heartbeat;

/// Parsed MAVLink ingress events for the Recorder Domain.
#[derive(Clone, Debug, PartialEq)]
pub enum MavlinkFact {
    /// Full current armed state from the vehicle heartbeat.
    ArmedChanged(bool),
    /// A camera component sent its first heartbeat.
    CameraHeartbeat(SystemAndComponent),
    /// Whether the camera can capture video.
    CameraRecordingCapability {
        /// Camera ids.
        camera: SystemAndComponent,
        /// Whether video capture is supported.
        capture_video: bool,
    },
    /// A video stream was registered on the backbone.
    VideoStreamRegistered {
        /// `video/...` topic.
        topic: String,
        /// Owning camera.
        camera: SystemAndComponent,
    },
    /// A capture command from the camera manager.
    CameraCaptureCommand {
        /// Capture command kind.
        command: MavlinkCaptureCommand,
        /// System and component that sent the command, which the `COMMAND_ACK` is addressed to.
        sender: SystemAndComponent,
        /// Target system.
        target_system: u8,
        /// Target component.
        target_component: u8,
        /// Requested status rate in hertz (`param2` on the command).
        status_interval_hertz: f32,
    },
}

/// Capture commands the camera manager sends over MAVLink.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MavlinkCaptureCommand {
    /// Start video capture.
    StartCapture,
    /// Stop video capture.
    StopCapture,
    /// Request capture status.
    RequestCaptureStatus,
}

/// Discovery messages requested after a camera heartbeat.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MavlinkDiscoveryMessage {
    /// Request `CAMERA_INFORMATION`.
    CameraInformation,
    /// Request `VIDEO_STREAM_INFORMATION`.
    VideoStreamInformation,
}

/// Stateful ingress parser (armed dedup for change detection; periodic resend is the Task's job).
#[derive(Default)]
pub struct MavlinkIngressState {
    vehicle_armed: Option<bool>,
    known_cameras: BTreeSet<SystemAndComponent>,
}

/// A MAVLink system and component id pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct SystemAndComponent {
    /// MAVLink system id.
    pub system_id: u8,
    /// MAVLink component id.
    pub component_id: u8,
}

/// Parses one raw MAVLink frame into zero or more [`MavlinkFact`] values.
pub fn facts_from_frame(state: &mut MavlinkIngressState, bytes: &[u8]) -> Vec<MavlinkFact> {
    let Some(packet) = PacketRef::new(bytes) else {
        trace!("Not a MAVLink frame");
        return Vec::new();
    };
    if !is_handled_message(packet.message_id()) {
        return Vec::new();
    }

    match packet.message_id() {
        id if id == HEARTBEAT_DATA::ID => heartbeat_facts(state, &packet),
        id if id == CAMERA_INFORMATION_DATA::ID => camera_information_facts(&packet),
        id if id == VIDEO_STREAM_INFORMATION_DATA::ID => video_stream_facts(&packet),
        id if id == COMMAND_LONG_DATA::ID => command_long_facts(&packet),
        _ => Vec::new(),
    }
}

/// Whether this adapter parses the given MAVLink message id.
pub fn is_handled_message(message_id: u32) -> bool {
    message_id == HEARTBEAT_DATA::ID
        || message_id == CAMERA_INFORMATION_DATA::ID
        || message_id == VIDEO_STREAM_INFORMATION_DATA::ID
        || message_id == COMMAND_LONG_DATA::ID
}

/// Latest armed state held by the ingress parser (for periodic Observed facts).
pub fn last_vehicle_armed(state: &MavlinkIngressState) -> Option<bool> {
    state.vehicle_armed
}

/// Default source ids for discovery commands to cameras.
pub fn default_discovery_source() -> SystemAndComponent {
    SystemAndComponent {
        system_id: 255,
        component_id: MavComponent::MAV_COMP_ID_MISSIONPLANNER as u8,
    }
}

fn decode<D: MessageData>(packet: &PacketRef) -> Option<D> {
    if packet.try_validate::<MavMessage>().is_err() {
        warn!(msg_id = packet.message_id(), "Frame failed CRC validation");
        return None;
    }
    let version = match packet {
        PacketRef::V1(_) => mavlink::MavlinkVersion::V1,
        PacketRef::V2(_) => mavlink::MavlinkVersion::V2,
    };
    match D::deser(version, packet.payload()) {
        Ok(data) => Some(data),
        Err(error) => {
            warn!(%error, msg_id = packet.message_id(), "Failed decoding frame");
            None
        }
    }
}

fn heartbeat_facts(state: &mut MavlinkIngressState, packet: &PacketRef) -> Vec<MavlinkFact> {
    let Some(data) = decode::<HEARTBEAT_DATA>(packet) else {
        return Vec::new();
    };
    let source = SystemAndComponent {
        system_id: *packet.system_id(),
        component_id: *packet.component_id(),
    };
    if source.component_id == MavComponent::MAV_COMP_ID_AUTOPILOT1 as u8 {
        let armed = armed_from_heartbeat(&data);
        if state.vehicle_armed != Some(armed) {
            state.vehicle_armed = Some(armed);
            return vec![MavlinkFact::ArmedChanged(armed)];
        }
        return Vec::new();
    }
    if data.mavtype == MavType::MAV_TYPE_CAMERA && state.known_cameras.insert(source) {
        return vec![MavlinkFact::CameraHeartbeat(source)];
    }
    Vec::new()
}

fn camera_information_facts(packet: &PacketRef) -> Vec<MavlinkFact> {
    let Some(data) = decode::<CAMERA_INFORMATION_DATA>(packet) else {
        return Vec::new();
    };
    let camera = SystemAndComponent {
        system_id: *packet.system_id(),
        component_id: *packet.component_id(),
    };
    let capture_video = data
        .flags
        .contains(mavlink::dialects::ardupilotmega::CameraCapFlags::CAMERA_CAP_FLAGS_CAPTURE_VIDEO);
    vec![MavlinkFact::CameraRecordingCapability {
        camera,
        capture_video,
    }]
}

fn video_stream_facts(packet: &PacketRef) -> Vec<MavlinkFact> {
    let Some(data) = decode::<VIDEO_STREAM_INFORMATION_DATA>(packet) else {
        return Vec::new();
    };
    let camera = SystemAndComponent {
        system_id: *packet.system_id(),
        component_id: *packet.component_id(),
    };
    let name = data.name.to_str().unwrap_or("");
    if name.is_empty() {
        return Vec::new();
    }
    vec![MavlinkFact::VideoStreamRegistered {
        topic: video_topic_from_name(name),
        camera,
    }]
}

fn command_long_facts(packet: &PacketRef) -> Vec<MavlinkFact> {
    let Some(data) = decode::<COMMAND_LONG_DATA>(packet) else {
        return Vec::new();
    };
    let Some(command) = mavlink_capture_command(data.command) else {
        return Vec::new();
    };
    vec![MavlinkFact::CameraCaptureCommand {
        command,
        sender: SystemAndComponent {
            system_id: *packet.system_id(),
            component_id: *packet.component_id(),
        },
        target_system: data.target_system,
        target_component: data.target_component,
        status_interval_hertz: data.param2,
    }]
}

fn mavlink_capture_command(command: MavCmd) -> Option<MavlinkCaptureCommand> {
    match command {
        MavCmd::MAV_CMD_VIDEO_START_CAPTURE => Some(MavlinkCaptureCommand::StartCapture),
        MavCmd::MAV_CMD_VIDEO_STOP_CAPTURE => Some(MavlinkCaptureCommand::StopCapture),
        #[expect(
            deprecated,
            reason = "camera manager still sends REQUEST_CAMERA_CAPTURE_STATUS"
        )]
        MavCmd::MAV_CMD_REQUEST_CAMERA_CAPTURE_STATUS => {
            Some(MavlinkCaptureCommand::RequestCaptureStatus)
        }
        _ => None,
    }
}
