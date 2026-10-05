//! Capture command acceptance, rejection, and acknowledgements.

mod common;

use blueos_domain::Effect;
use blueos_recorder_cameras::{
    Cameras, CamerasIoRequest, CamerasTimerKey, CaptureCommandKind, VIDEO_CAPTURE_STATUS_IDLE,
};

use common::{GROUND_STATION, camera_capture_command, enable_capture_video, register_stream};

#[test]
fn start_capture_is_rejected_without_capture_video_capability() {
    let mut cameras = Cameras::default();
    register_stream(&mut cameras, "video/front/stream", 1, 100);
    let effects = camera_capture_command(&mut cameras, CaptureCommandKind::StartCapture, 0);
    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::Io(CamerasIoRequest::CommandAck {
            command: CaptureCommandKind::StartCapture,
            accepted: false,
            ..
        })
    )));
    assert!(
        !cameras
            .video_recording_by_topic()
            .any(|(_, recording)| recording),
        "stream must not enter recording without capability"
    );
}

#[test]
fn stop_capture_publishes_one_idle_capture_status() {
    let mut cameras = Cameras::default();
    register_stream(&mut cameras, "video/front/stream", 1, 100);
    enable_capture_video(&mut cameras, 1, 100);
    camera_capture_command(&mut cameras, CaptureCommandKind::StartCapture, 10);

    let effects = camera_capture_command(&mut cameras, CaptureCommandKind::StopCapture, 12);

    let idle_statuses = effects
        .iter()
        .filter(|effect| {
            matches!(
                effect,
                Effect::Io(CamerasIoRequest::CaptureStatus {
                    video_status: VIDEO_CAPTURE_STATUS_IDLE,
                    recording_time_ms: 0,
                    ..
                })
            )
        })
        .count();
    assert_eq!(
        idle_statuses, 1,
        "stop must tell the sender the stream is idle"
    );
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            Effect::Cancel(CamerasTimerKey::CaptureStatus { .. })
        )),
        "stop must still cancel the periodic capture status timer"
    );
}

#[test]
fn command_ack_is_addressed_to_the_sender_of_each_capture_command() {
    let mut cameras = Cameras::default();
    register_stream(&mut cameras, "video/front/stream", 1, 100);
    enable_capture_video(&mut cameras, 1, 100);

    for command in [
        CaptureCommandKind::StartCapture,
        CaptureCommandKind::RequestCaptureStatus,
        CaptureCommandKind::StopCapture,
    ] {
        let effects = camera_capture_command(&mut cameras, command, 1);
        assert!(
            effects.iter().any(|effect| matches!(
                effect,
                Effect::Io(CamerasIoRequest::CommandAck { recipient, .. })
                    if *recipient == GROUND_STATION
            )),
            "{command:?} must be acknowledged to the sender"
        );
    }
}

#[test]
fn start_capture_is_accepted_without_evidence_that_the_stream_publishes() {
    let mut cameras = Cameras::default();
    register_stream(&mut cameras, "video/fakesource/stream", 1, 100);
    enable_capture_video(&mut cameras, 1, 100);

    let effects = camera_capture_command(&mut cameras, CaptureCommandKind::StartCapture, 1);

    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::Io(CamerasIoRequest::CommandAck { accepted: true, .. })
    )));
    assert!(
        cameras
            .video_recording_by_topic()
            .any(|(topic, recording)| topic == "video/fakesource/stream" && recording)
    );
}
