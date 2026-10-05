//! Shared helpers for cameras integration tests.

#![expect(
    dead_code,
    reason = "each integration test binary uses only a subset of these helpers"
)]

use core::time::Duration;

use blueos_domain::{Effect, Now, Outcome};
use blueos_recorder_cameras::{
    Cameras, CamerasIoRequest, CamerasObservedFact, CamerasTimerKey, CaptureCommandKind,
    SystemAndComponent,
};

pub(crate) const GROUND_STATION: SystemAndComponent = SystemAndComponent {
    system_id: 255,
    component_id: 190,
};

pub(crate) const fn now_at(monotonic_seconds: u64) -> Now {
    Now {
        wall: Duration::from_secs(1_700_000_000),
        monotonic: Duration::from_secs(monotonic_seconds),
    }
}

pub(crate) fn register_stream(cameras: &mut Cameras, topic: &str, system_id: u8, component_id: u8) {
    let outcome = cameras.handle_observed_fact(
        CamerasObservedFact::RegisterVideoStream {
            topic: topic.into(),
            camera: SystemAndComponent {
                system_id,
                component_id,
            },
        },
        now_at(0),
    );
    assert!(matches!(outcome, Outcome::Applied { .. }));
}

pub(crate) fn enable_capture_video(cameras: &mut Cameras, system_id: u8, component_id: u8) {
    let outcome = cameras.handle_observed_fact(
        CamerasObservedFact::SetCameraRecordingCapability {
            camera: SystemAndComponent {
                system_id,
                component_id,
            },
            capture_video: true,
        },
        now_at(0),
    );
    assert!(matches!(outcome, Outcome::Applied { .. }));
}

pub(crate) fn camera_capture_command(
    cameras: &mut Cameras,
    command: CaptureCommandKind,
    monotonic_seconds: u64,
) -> Vec<Effect<blueos_recorder_cameras::CamerasTick, CamerasIoRequest, CamerasTimerKey>> {
    let outcome = cameras.handle_observed_fact(
        CamerasObservedFact::CameraCaptureCommand {
            command,
            sender: GROUND_STATION,
            target_system: 1,
            target_component: 100,
            status_interval: Duration::from_secs(1),
        },
        now_at(monotonic_seconds),
    );
    let Outcome::Applied { effects, .. } = outcome else {
        panic!("capture command must apply");
    };
    effects
}

pub(crate) fn start_capture_for_component(
    cameras: &mut Cameras,
    target_system: u8,
    target_component: u8,
    status_interval: Duration,
    monotonic_seconds: u64,
) -> Vec<Effect<blueos_recorder_cameras::CamerasTick, CamerasIoRequest, CamerasTimerKey>> {
    let outcome = cameras.handle_observed_fact(
        CamerasObservedFact::CameraCaptureCommand {
            command: CaptureCommandKind::StartCapture,
            sender: GROUND_STATION,
            target_system,
            target_component,
            status_interval,
        },
        now_at(monotonic_seconds),
    );
    let Outcome::Applied { effects, .. } = outcome else {
        panic!("start capture must apply");
    };
    effects
}

pub(crate) fn effects_schedule_capture_status_for_topic(
    effects: &[Effect<blueos_recorder_cameras::CamerasTick, CamerasIoRequest, CamerasTimerKey>],
    topic: &str,
) -> bool {
    effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Schedule {
                key: CamerasTimerKey::CaptureStatus { topic: scheduled_topic },
                ..
            } if scheduled_topic == topic
        )
    })
}

pub(crate) fn count_capture_status_io_effects(
    effects: &[Effect<blueos_recorder_cameras::CamerasTick, CamerasIoRequest, CamerasTimerKey>],
) -> usize {
    effects
        .iter()
        .filter(|effect| matches!(effect, Effect::Io(CamerasIoRequest::CaptureStatus { .. })))
        .count()
}
