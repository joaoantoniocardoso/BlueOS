//! Effect builders for MAVLink capture commands on one video stream.

use alloc::{string::String, vec::Vec};
use core::time::Duration;

use blueos_domain::{Effect, Now};
use blueos_recorder_capture::Capture;

use crate::types::{
    CamerasIoRequest, CamerasTick, CamerasTimerKey, CaptureCommandInput, StartCaptureDispatch,
    VIDEO_CAPTURE_STATUS_IDLE, VIDEO_CAPTURE_STATUS_RECORDING, VideoStream,
};

pub(crate) fn push_start_capture_effects(
    stream: &mut VideoStream,
    topic: String,
    dispatch: &StartCaptureDispatch<'_>,
    effects: &mut Vec<Effect<CamerasTick, CamerasIoRequest, CamerasTimerKey>>,
) {
    let camera = stream.camera;
    effects.push(Effect::Io(CamerasIoRequest::CommandAck {
        camera,
        recipient: dispatch.input.sender,
        command: dispatch.input.command,
        accepted: dispatch.accepted,
    }));
    if !dispatch.accepted {
        return;
    }
    stream.is_recording = true;
    stream.recording_started_at = Some(dispatch.now.monotonic);
    stream.status_interval = dispatch.interval;
    let timer_key = CamerasTimerKey::CaptureStatus {
        topic: topic.clone(),
    };
    effects.push(Effect::Cancel(timer_key.clone()));
    effects.push(Effect::Schedule {
        after: Duration::ZERO,
        key: timer_key,
        command: CamerasTick::CaptureStatus { topic },
    });
}

pub(crate) fn push_stop_capture_effects(
    stream: &mut VideoStream,
    topic: String,
    input: &CaptureCommandInput,
    effects: &mut Vec<Effect<CamerasTick, CamerasIoRequest, CamerasTimerKey>>,
) {
    let camera = stream.camera;
    stream.is_recording = false;
    stream.recording_started_at = None;
    effects.push(Effect::Io(CamerasIoRequest::CommandAck {
        camera,
        recipient: input.sender,
        command: input.command,
        accepted: true,
    }));
    effects.push(Effect::Io(CamerasIoRequest::CaptureStatus {
        camera,
        video_status: VIDEO_CAPTURE_STATUS_IDLE,
        recording_time_ms: 0,
    }));
    effects.push(Effect::Cancel(CamerasTimerKey::CaptureStatus { topic }));
}

pub(crate) fn push_request_capture_status_effects(
    stream: &VideoStream,
    input: &CaptureCommandInput,
    now: Now,
    effects: &mut Vec<Effect<CamerasTick, CamerasIoRequest, CamerasTimerKey>>,
) {
    let camera = stream.camera;
    let (video_status, recording_time_ms) = capture_status_fields(stream, now);
    effects.push(Effect::Io(CamerasIoRequest::CommandAck {
        camera,
        recipient: input.sender,
        command: input.command,
        accepted: true,
    }));
    effects.push(Effect::Io(CamerasIoRequest::CaptureStatus {
        camera,
        video_status,
        recording_time_ms,
    }));
}

pub(crate) fn capture_status_fields(stream: &VideoStream, now: Now) -> (u8, u32) {
    if stream.is_recording {
        let recording_time_ms = stream
            .recording_started_at
            .map(|started_at| Capture::recording_time_ms(started_at, now))
            .unwrap_or(0);
        (VIDEO_CAPTURE_STATUS_RECORDING, recording_time_ms)
    } else {
        (VIDEO_CAPTURE_STATUS_IDLE, 0)
    }
}

pub(crate) fn is_addressed_to(
    target_system: u8,
    target_component: u8,
    camera_system: u8,
    camera_component: u8,
) -> bool {
    let system_matches = target_system == 0 || target_system == camera_system;
    let component_matches = target_component == 0 || target_component == camera_component;
    system_matches && component_matches
}
