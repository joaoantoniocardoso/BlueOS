//! L1 tests for the MAVLink capture command ingress and the `COMMAND_ACK` reply.

use mavlink::{
    MavlinkVersion, MessageData,
    dialects::ardupilotmega::{COMMAND_ACK_DATA, MavCmd},
};
use mavlink_codec::PacketRef;

use blueos_recorder_cameras::CaptureCommandKind;
use blueos_recorder_mavlink::{
    MavlinkFact, MavlinkIngressState, SystemAndComponent, build_command_ack, encode_command_long,
    facts_from_frame,
};

const GROUND_STATION: SystemAndComponent = SystemAndComponent {
    system_id: 255,
    component_id: 190,
};
const CAMERA: SystemAndComponent = SystemAndComponent {
    system_id: 1,
    component_id: 101,
};

#[test]
fn command_ack_is_addressed_to_the_sender_of_the_command() {
    let frame = build_command_ack(
        CAMERA,
        GROUND_STATION,
        0,
        CaptureCommandKind::StartCapture,
        true,
    );

    let packet = PacketRef::new(&frame).expect("frame must parse");
    let ack = COMMAND_ACK_DATA::deser(MavlinkVersion::V2, packet.payload())
        .expect("payload must be a COMMAND_ACK");
    assert_eq!(
        (ack.target_system, ack.target_component),
        (GROUND_STATION.system_id, GROUND_STATION.component_id)
    );
}

#[test]
fn capture_command_fact_carries_the_sender() {
    let mut sequence = 0;
    let frame = encode_command_long(
        GROUND_STATION,
        &mut sequence,
        CAMERA,
        MavCmd::MAV_CMD_VIDEO_START_CAPTURE,
        [0.0; 7],
    );

    let facts = facts_from_frame(&mut MavlinkIngressState::default(), &frame);

    let [
        MavlinkFact::CameraCaptureCommand {
            sender,
            target_system,
            target_component,
            ..
        },
    ] = facts.as_slice()
    else {
        panic!("expected one capture command fact, got {facts:?}");
    };
    assert_eq!(*sender, GROUND_STATION);
    assert_eq!(
        (*target_system, *target_component),
        (CAMERA.system_id, CAMERA.component_id)
    );
}
