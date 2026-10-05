use mavlink::{
    MavHeader, MessageData,
    dialects::ardupilotmega::{
        CAMERA_CAPTURE_STATUS_DATA, CAMERA_INFORMATION_DATA, COMMAND_ACK_DATA, COMMAND_LONG_DATA,
        HEARTBEAT_DATA, MavCmd, MavMessage, MavModeFlag, MavResult, MavState, MavType,
        VIDEO_STREAM_INFORMATION_DATA,
    },
};

use blueos_recorder_cameras::CaptureCommandKind;

use crate::{MavlinkDiscoveryMessage, SystemAndComponent, mavlink_command_for_capture};

/// `COMMAND_LONG` carries seven `f32` parameters (`param1`..`param7`).
const COMMAND_LONG_PARAMETER_COUNT: usize = 7;
/// MAVLink v2 wire protocol version field on `HEARTBEAT` (`mavlink_version`).
const MAVLINK_V2_WIRE_PROTOCOL_VERSION: u8 = 3;
type CommandLongParameters = [f32; COMMAND_LONG_PARAMETER_COUNT];

/// Builds a `COMMAND_LONG` discovery request for a camera.
pub fn build_discovery_request(
    source: SystemAndComponent,
    sequence: &mut u8,
    camera: SystemAndComponent,
    message: MavlinkDiscoveryMessage,
) -> Vec<u8> {
    let message_id = match message {
        MavlinkDiscoveryMessage::CameraInformation => CAMERA_INFORMATION_DATA::ID as f32,
        MavlinkDiscoveryMessage::VideoStreamInformation => VIDEO_STREAM_INFORMATION_DATA::ID as f32,
    };
    encode_command_long(
        source,
        sequence,
        camera,
        MavCmd::MAV_CMD_REQUEST_MESSAGE,
        [message_id, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    )
}

// qual:test_helper
/// Encodes a ground station's video start (or, when `start` is false, stop) capture command to a camera, for tests
/// (`mavlink_raw/out` ingress).
pub fn test_camera_capture_frame(start: bool, target_system: u8, target_component: u8) -> Vec<u8> {
    let command = if start {
        MavCmd::MAV_CMD_VIDEO_START_CAPTURE
    } else {
        MavCmd::MAV_CMD_VIDEO_STOP_CAPTURE
    };
    encode_command_long(
        crate::default_discovery_source(),
        &mut 0,
        SystemAndComponent {
            system_id: target_system,
            component_id: target_component,
        },
        command,
        [0.0; COMMAND_LONG_PARAMETER_COUNT],
    )
}

/// Encodes a `COMMAND_LONG` frame.
pub fn encode_command_long(
    source: SystemAndComponent,
    sequence: &mut u8,
    target: SystemAndComponent,
    command: MavCmd,
    parameters: CommandLongParameters,
) -> Vec<u8> {
    let header = MavHeader {
        system_id: source.system_id,
        component_id: source.component_id,
        sequence: {
            let value = *sequence;
            *sequence = sequence.wrapping_add(1);
            value
        },
    };
    let message = MavMessage::COMMAND_LONG(COMMAND_LONG_DATA {
        target_system: target.system_id,
        target_component: target.component_id,
        command,
        confirmation: 0,
        param1: parameters[0],
        param2: parameters[1],
        param3: parameters[2],
        param4: parameters[3],
        param5: parameters[4],
        param6: parameters[5],
        param7: parameters[6],
    });
    encode_mavlink_message(header, &message)
}

/// Builds a `COMMAND_ACK` frame for a capture command, addressed to the `recipient` that sent it.
pub fn build_command_ack(
    camera: SystemAndComponent,
    recipient: SystemAndComponent,
    sequence: u8,
    command: CaptureCommandKind,
    accepted: bool,
) -> Vec<u8> {
    let result = if accepted {
        MavResult::MAV_RESULT_ACCEPTED
    } else {
        MavResult::MAV_RESULT_DENIED
    };
    let header = MavHeader {
        system_id: camera.system_id,
        component_id: camera.component_id,
        sequence,
    };
    let mavlink_command = mavlink_command_for_capture(command);
    encode_mavlink_message(
        header,
        &MavMessage::COMMAND_ACK(COMMAND_ACK_DATA {
            command: mavlink_command,
            result,
            progress: u8::MAX,
            result_param2: 0,
            target_system: recipient.system_id,
            target_component: recipient.component_id,
        }),
    )
}

/// Builds a `CAMERA_CAPTURE_STATUS` frame.
pub fn build_camera_capture_status(
    camera: SystemAndComponent,
    sequence: u8,
    video_status: u8,
    recording_time_ms: u32,
) -> Vec<u8> {
    let header = MavHeader {
        system_id: camera.system_id,
        component_id: camera.component_id,
        sequence,
    };
    encode_mavlink_message(
        header,
        &MavMessage::CAMERA_CAPTURE_STATUS(CAMERA_CAPTURE_STATUS_DATA {
            time_boot_ms: 0,
            image_interval: 0.0,
            recording_time_ms,
            available_capacity: 0.0,
            image_status: 0,
            video_status,
            image_count: 0,
            camera_device_id: 0,
        }),
    )
}

// qual:test_helper
/// Encodes an autopilot heartbeat for tests (`mavlink_raw/out` ingress).
pub fn test_vehicle_heartbeat_frame(armed: bool) -> Vec<u8> {
    let mut base_mode = MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED;
    if armed {
        base_mode.insert(MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED);
    }
    let header = MavHeader {
        system_id: 1,
        component_id: mavlink::dialects::ardupilotmega::MavComponent::MAV_COMP_ID_AUTOPILOT1 as u8,
        sequence: 0,
    };
    encode_mavlink_message(
        header,
        &MavMessage::HEARTBEAT(HEARTBEAT_DATA {
            custom_mode: 0,
            mavtype: MavType::MAV_TYPE_SUBMARINE,
            autopilot: mavlink::dialects::ardupilotmega::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
            base_mode,
            system_status: MavState::MAV_STATE_ACTIVE,
            mavlink_version: MAVLINK_V2_WIRE_PROTOCOL_VERSION,
        }),
    )
}

/// Encodes a MAVLink v2 frame.
pub fn encode_mavlink_message(header: MavHeader, message: &MavMessage) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Err(error) = mavlink::write_v2_msg(&mut bytes, header, message) {
        tracing::warn!(%error, "Failed to encode MAVLink message");
    }
    bytes
}
