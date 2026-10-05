//! Independent capture status timers per camera stream.

mod common;

use core::time::Duration;

use blueos_domain::{Effect, Outcome};
use blueos_recorder_cameras::{Cameras, CamerasIoRequest, CamerasTick};

use common::{
    count_capture_status_io_effects, effects_schedule_capture_status_for_topic,
    enable_capture_video, now_at, register_stream, start_capture_for_component,
};

#[test]
fn two_cameras_get_independent_capture_status_timers() {
    let mut cameras = Cameras::default();
    register_stream(&mut cameras, "video/front/stream", 1, 100);
    register_stream(&mut cameras, "video/rear/stream", 1, 101);
    enable_capture_video(&mut cameras, 1, 100);
    enable_capture_video(&mut cameras, 1, 101);

    let front_effects =
        start_capture_for_component(&mut cameras, 1, 100, Duration::from_millis(500), 1);
    assert!(
        effects_schedule_capture_status_for_topic(&front_effects, "video/front/stream"),
        "front camera must arm its own capture status timer"
    );

    let rear_effects =
        start_capture_for_component(&mut cameras, 1, 101, Duration::from_millis(500), 1);
    assert!(
        effects_schedule_capture_status_for_topic(&rear_effects, "video/rear/stream"),
        "rear camera must arm its own capture status timer"
    );

    let front_tick = cameras.handle_tick(
        CamerasTick::CaptureStatus {
            topic: "video/front/stream".into(),
        },
        now_at(2),
    );
    let Outcome::Applied {
        effects: front_tick_effects,
        ..
    } = front_tick
    else {
        panic!("front tick must apply");
    };
    assert_eq!(
        count_capture_status_io_effects(&front_tick_effects),
        1,
        "front tick must publish one capture status"
    );

    let rear_tick = cameras.handle_tick(
        CamerasTick::CaptureStatus {
            topic: "video/rear/stream".into(),
        },
        now_at(2),
    );
    let Outcome::Applied {
        effects: rear_tick_effects,
        ..
    } = rear_tick
    else {
        panic!("rear tick must apply");
    };
    assert_eq!(
        count_capture_status_io_effects(&rear_tick_effects),
        1,
        "rear tick must publish one capture status"
    );
}

#[test]
fn capture_status_uses_recording_time_ms_from_monotonic_clock() {
    let mut cameras = Cameras::default();
    register_stream(&mut cameras, "video/front/stream", 1, 100);
    enable_capture_video(&mut cameras, 1, 100);
    start_capture_for_component(&mut cameras, 1, 100, Duration::from_secs(1), 10);

    let tick = cameras.handle_tick(
        CamerasTick::CaptureStatus {
            topic: "video/front/stream".into(),
        },
        now_at(12),
    );
    let Outcome::Applied { effects, .. } = tick else {
        panic!("tick must apply");
    };
    let Some(Effect::Io(CamerasIoRequest::CaptureStatus {
        recording_time_ms, ..
    })) = effects
        .iter()
        .find(|effect| matches!(effect, Effect::Io(CamerasIoRequest::CaptureStatus { .. })))
    else {
        panic!("expected capture status IO");
    };
    assert_eq!(*recording_time_ms, 2_000);
}
