//! Camera heartbeat and MAVLink discovery requests.

mod common;

use blueos_domain::{Effect, Outcome};
use blueos_recorder_cameras::{
    Cameras, CamerasIoRequest, CamerasObservedFact, DiscoveryMessageKind, SystemAndComponent,
};

use common::now_at;

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
