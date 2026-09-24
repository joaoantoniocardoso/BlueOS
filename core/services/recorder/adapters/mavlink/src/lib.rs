//! Raw MAVLink parse and serialize for the recorder (MCM external recorder contract).

extern crate alloc;

mod camera;
mod encode;
mod vehicle;

use mavlink::MessageData;
use mavlink::dialects::ardupilotmega::{
    CAMERA_INFORMATION_DATA, COMMAND_LONG_DATA, HEARTBEAT_DATA, MavCmd, MavComponent, MavMessage,
    MavType, VIDEO_STREAM_INFORMATION_DATA,
};
use mavlink_codec::PacketRef;
use tracing::{trace, warn};

pub use camera::video_topic_from_name;
pub use encode::{
    build_camera_capture_status, build_command_ack, encode_command_long, encode_mavlink_message,
};
pub use vehicle::ArmState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct SystemAndComponent {
    pub system_id: u8,
    pub component_id: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MavlinkFact {
    ArmedChanged(bool),
    CameraHeartbeat(SystemAndComponent),
    CameraRecordingCapability {
        camera: SystemAndComponent,
        capture_video: bool,
    },
    VideoStreamRegistered {
        topic: String,
        camera: SystemAndComponent,
    },
    CameraCaptureCommand {
        command: MavCmd,
        target_system: u8,
        target_component: u8,
        status_interval_hertz: f32,
    },
}

pub fn is_handled_message(message_id: u32) -> bool {
    message_id == HEARTBEAT_DATA::ID
        || message_id == CAMERA_INFORMATION_DATA::ID
        || message_id == VIDEO_STREAM_INFORMATION_DATA::ID
        || message_id == COMMAND_LONG_DATA::ID
}

#[derive(Default)]
pub struct MavlinkIngressState {
    vehicle_armed: Option<bool>,
    known_cameras: alloc::collections::BTreeSet<SystemAndComponent>,
}

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
        if let Some(armed) = vehicle::armed_from_heartbeat(&data)
            && state.vehicle_armed != Some(armed)
        {
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

#[allow(deprecated)]
fn command_long_facts(packet: &PacketRef) -> Vec<MavlinkFact> {
    let Some(data) = decode::<COMMAND_LONG_DATA>(packet) else {
        return Vec::new();
    };
    match data.command {
        MavCmd::MAV_CMD_VIDEO_START_CAPTURE
        | MavCmd::MAV_CMD_VIDEO_STOP_CAPTURE
        | MavCmd::MAV_CMD_REQUEST_CAMERA_CAPTURE_STATUS => {
            vec![MavlinkFact::CameraCaptureCommand {
                command: data.command,
                target_system: data.target_system,
                target_component: data.target_component,
                status_interval_hertz: data.param2,
            }]
        }
        _ => Vec::new(),
    }
}

pub fn discovery_requests_for_camera(
    source: SystemAndComponent,
    sequence: &mut u8,
    camera: SystemAndComponent,
) -> Vec<Vec<u8>> {
    vec![
        encode_command_long(
            source,
            sequence,
            camera,
            MavCmd::MAV_CMD_REQUEST_MESSAGE,
            [
                CAMERA_INFORMATION_DATA::ID as f32,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ],
        ),
        encode_command_long(
            source,
            sequence,
            camera,
            MavCmd::MAV_CMD_REQUEST_MESSAGE,
            [
                VIDEO_STREAM_INFORMATION_DATA::ID as f32,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ],
        ),
    ]
}

pub fn default_discovery_source() -> SystemAndComponent {
    SystemAndComponent {
        system_id: 255,
        component_id: MavComponent::MAV_COMP_ID_MISSIONPLANNER as u8,
    }
}
