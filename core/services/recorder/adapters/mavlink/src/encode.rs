use mavlink::{
    MavHeader,
    dialects::ardupilotmega::{
        CAMERA_CAPTURE_STATUS_DATA, COMMAND_ACK_DATA, COMMAND_LONG_DATA, MavCmd, MavMessage,
        MavResult,
    },
};

use crate::SystemAndComponent;

pub fn encode_mavlink_message(header: MavHeader, message: &MavMessage) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Err(error) = mavlink::write_v2_msg(&mut bytes, header, message) {
        tracing::warn!(%error, "Failed to encode MAVLink message");
    }
    bytes
}

pub fn encode_command_long(
    source: SystemAndComponent,
    sequence: &mut u8,
    target: SystemAndComponent,
    command: MavCmd,
    parameters: [f32; 7],
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

#[allow(deprecated)]
pub fn build_command_ack(
    camera: SystemAndComponent,
    sequence: u8,
    command: u32,
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
    let mavlink_command = match command {
        2500 => MavCmd::MAV_CMD_VIDEO_START_CAPTURE,
        2501 => MavCmd::MAV_CMD_VIDEO_STOP_CAPTURE,
        522 => MavCmd::MAV_CMD_REQUEST_CAMERA_CAPTURE_STATUS,
        _ => MavCmd::MAV_CMD_REQUEST_CAMERA_CAPTURE_STATUS,
    };
    encode_mavlink_message(
        header,
        &MavMessage::COMMAND_ACK(COMMAND_ACK_DATA {
            command: mavlink_command,
            result,
        }),
    )
}

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
        }),
    )
}
