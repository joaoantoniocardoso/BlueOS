//! MAVLink encoding and publish for cameras Block IO Effects.

use core::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use tracing::warn;

use blueos_comms::{Payload, Sample};
use blueos_domain::IoError;
use blueos_recorder_cameras::{
    CamerasIoRequest, CaptureCommandKind, DiscoveryMessageKind, RAW_MAVLINK_IN_TOPIC,
};
use blueos_recorder_domain::RecorderIoResult;
use blueos_recorder_mavlink::{
    MavlinkCaptureCommand, MavlinkDiscoveryMessage, SystemAndComponent as MavlinkSystem,
    build_camera_capture_status, build_command_ack, build_discovery_request,
    default_discovery_source,
};
use blueos_service::Session;

/// Encodes and publishes one cameras IO request on the backbone session.
pub(crate) async fn publish_cameras_request(
    session: Session,
    mavlink_sequence: Arc<AtomicU8>,
    request: CamerasIoRequest,
) -> Result<Option<RecorderIoResult>, IoError> {
    let bytes = encode_cameras_io(&mavlink_sequence, request);
    session
        .publish(Sample::new(
            RAW_MAVLINK_IN_TOPIC,
            Payload::new(Bytes::from(bytes)),
            "application/octet-stream",
        ))
        .await
        .map_err(|error| {
            warn!(
                %error,
                topic = RAW_MAVLINK_IN_TOPIC,
                "Failed to publish cameras MAVLink request"
            );
            IoError::new("failed to publish cameras MAVLink request")
        })?;
    Ok(None)
}

fn encode_cameras_io(mavlink_sequence: &Arc<AtomicU8>, request: CamerasIoRequest) -> Vec<u8> {
    let discovery_source = default_discovery_source();
    match request {
        CamerasIoRequest::CommandAck {
            camera,
            command,
            accepted,
        } => build_command_ack(
            to_mavlink_system(camera),
            next_sequence(mavlink_sequence),
            to_mavlink_capture_command(command),
            accepted,
        ),
        CamerasIoRequest::CaptureStatus {
            camera,
            video_status,
            recording_time_ms,
        } => build_camera_capture_status(
            to_mavlink_system(camera),
            next_sequence(mavlink_sequence),
            video_status,
            recording_time_ms,
        ),
        CamerasIoRequest::RequestDiscovery { camera, message } => {
            let mut sequence = mavlink_sequence.load(Ordering::Relaxed);
            let bytes = build_discovery_request(
                discovery_source,
                &mut sequence,
                to_mavlink_system(camera),
                to_mavlink_discovery_message(message),
            );
            mavlink_sequence.store(sequence, Ordering::Relaxed);
            bytes
        }
    }
}

fn next_sequence(mavlink_sequence: &Arc<AtomicU8>) -> u8 {
    mavlink_sequence.fetch_add(1, Ordering::Relaxed)
}

fn to_mavlink_system(camera: blueos_recorder_cameras::SystemAndComponent) -> MavlinkSystem {
    MavlinkSystem {
        system_id: camera.system_id,
        component_id: camera.component_id,
    }
}

fn to_mavlink_capture_command(command: CaptureCommandKind) -> MavlinkCaptureCommand {
    match command {
        CaptureCommandKind::StartCapture => MavlinkCaptureCommand::StartCapture,
        CaptureCommandKind::StopCapture => MavlinkCaptureCommand::StopCapture,
        CaptureCommandKind::RequestCaptureStatus => MavlinkCaptureCommand::RequestCaptureStatus,
    }
}

fn to_mavlink_discovery_message(message: DiscoveryMessageKind) -> MavlinkDiscoveryMessage {
    match message {
        DiscoveryMessageKind::CameraInformation => MavlinkDiscoveryMessage::CameraInformation,
        DiscoveryMessageKind::VideoStreamInformation => {
            MavlinkDiscoveryMessage::VideoStreamInformation
        }
    }
}
