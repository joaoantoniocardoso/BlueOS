//! L1 tests for the MAVLink camera protocol Block.

use core::time::Duration;

use blueos_domain::{Effect, Now, Outcome};
use blueos_recorder_cameras::{
    Cameras, CamerasIoRequest, CamerasObservedFact, CamerasTick, CamerasTimerKey,
    CaptureCommandKind, DiscoveryMessageKind, SystemAndComponent, VIDEO_CAPTURE_STATUS_IDLE,
};

const GROUND_STATION: SystemAndComponent = SystemAndComponent {
    system_id: 255,
    component_id: 190,
};

const fn now_at(monotonic_seconds: u64) -> Now {
    Now {
        wall: Duration::from_secs(1_700_000_000),
        monotonic: Duration::from_secs(monotonic_seconds),
    }
}

fn register_stream(cameras: &mut Cameras, topic: &str, system_id: u8, component_id: u8) {
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

fn enable_capture_video(cameras: &mut Cameras, system_id: u8, component_id: u8) {
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

#[test]
fn two_cameras_get_independent_capture_status_timers() {
    let mut cameras = Cameras::default();
    register_stream(&mut cameras, "video/front/stream", 1, 100);
    register_stream(&mut cameras, "video/rear/stream", 1, 101);
    enable_capture_video(&mut cameras, 1, 100);
    enable_capture_video(&mut cameras, 1, 101);

    let start_front = cameras.handle_observed_fact(
        CamerasObservedFact::CameraCaptureCommand {
            command: CaptureCommandKind::StartCapture,
            sender: GROUND_STATION,
            target_system: 1,
            target_component: 100,
            status_interval: Duration::from_millis(500),
        },
        now_at(1),
    );
    let Outcome::Applied {
        effects: front_effects,
        ..
    } = start_front
    else {
        panic!("start front must apply");
    };
    assert!(
        front_effects.iter().any(|effect| {
            matches!(
                effect,
                Effect::Schedule {
                    key: CamerasTimerKey::CaptureStatus { topic },
                    ..
                } if topic == "video/front/stream"
            )
        }),
        "front camera must arm its own capture status timer"
    );

    let start_rear = cameras.handle_observed_fact(
        CamerasObservedFact::CameraCaptureCommand {
            command: CaptureCommandKind::StartCapture,
            sender: GROUND_STATION,
            target_system: 1,
            target_component: 101,
            status_interval: Duration::from_millis(500),
        },
        now_at(1),
    );
    let Outcome::Applied {
        effects: rear_effects,
        ..
    } = start_rear
    else {
        panic!("start rear must apply");
    };
    assert!(
        rear_effects.iter().any(|effect| {
            matches!(
                effect,
                Effect::Schedule {
                    key: CamerasTimerKey::CaptureStatus { topic },
                    ..
                } if topic == "video/rear/stream"
            )
        }),
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
        front_tick_effects
            .iter()
            .filter(|effect| matches!(effect, Effect::Io(CamerasIoRequest::CaptureStatus { .. })))
            .count(),
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
        rear_tick_effects
            .iter()
            .filter(|effect| matches!(effect, Effect::Io(CamerasIoRequest::CaptureStatus { .. })))
            .count(),
        1,
        "rear tick must publish one capture status"
    );
}

#[test]
fn capture_status_uses_recording_time_ms_from_monotonic_clock() {
    let mut cameras = Cameras::default();
    register_stream(&mut cameras, "video/front/stream", 1, 100);
    enable_capture_video(&mut cameras, 1, 100);
    let start = cameras.handle_observed_fact(
        CamerasObservedFact::CameraCaptureCommand {
            command: CaptureCommandKind::StartCapture,
            sender: GROUND_STATION,
            target_system: 1,
            target_component: 100,
            status_interval: Duration::from_secs(1),
        },
        now_at(10),
    );
    assert!(matches!(start, Outcome::Applied { .. }));

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

#[test]
fn camera_heartbeat_schedules_discovery_requests() {
    let mut cameras = Cameras::default();
    let camera = SystemAndComponent {
        system_id: 1,
        component_id: 100,
    };
    let outcome =
        cameras.handle_observed_fact(CamerasObservedFact::CameraHeartbeat { camera }, now_at(0));
    let Outcome::Applied { effects, .. } = outcome else {
        panic!("heartbeat must apply");
    };
    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::Io(CamerasIoRequest::RequestDiscovery {
            message: DiscoveryMessageKind::CameraInformation,
            ..
        })
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::Io(CamerasIoRequest::RequestDiscovery {
            message: DiscoveryMessageKind::VideoStreamInformation,
            ..
        })
    )));
}

#[test]
fn start_capture_is_rejected_without_capture_video_capability() {
    let mut cameras = Cameras::default();
    register_stream(&mut cameras, "video/front/stream", 1, 100);
    let outcome = cameras.handle_observed_fact(
        CamerasObservedFact::CameraCaptureCommand {
            command: CaptureCommandKind::StartCapture,
            sender: GROUND_STATION,
            target_system: 1,
            target_component: 100,
            status_interval: Duration::from_secs(1),
        },
        now_at(0),
    );
    let Outcome::Applied { effects, .. } = outcome else {
        panic!("start must apply");
    };
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

fn camera_capture_command(
    cameras: &mut Cameras,
    command: CaptureCommandKind,
    monotonic_seconds: u64,
) -> Vec<Effect<CamerasTick, CamerasIoRequest, CamerasTimerKey>> {
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
