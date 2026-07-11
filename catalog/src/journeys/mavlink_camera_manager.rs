use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const GETTING_STARTED: &str = "content/usage/getting-started/index.md";
const VIDEO_MENUS: &str = "core/frontend/src/menus.ts";
const VIDEO_STORE: &str = "core/frontend/src/store/video.ts";
const VIDEO_MANAGER: &str = "core/frontend/src/components/video-manager/VideoManager.vue";
const VIDEO_DEVICE: &str = "core/frontend/src/components/video-manager/VideoDevice.vue";
const VIDEO_STREAM: &str = "core/frontend/src/components/video-manager/VideoStream.vue";
const VIDEO_STREAM_CREATION_DIALOG: &str =
    "core/frontend/src/components/video-manager/VideoStreamCreationDialog.vue";
const VIDEO_CONTROLS_DIALOG: &str =
    "core/frontend/src/components/video-manager/VideoControlsDialog.vue";
const RUNTIME_CAPTURE: &str = "runtime-captures/mavlink_camera_manager__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn journeys() -> Vec<UserJourney> {
    vec![
        view_camera_streams(),
        configure_camera_stream(),
        remove_camera_stream(),
        configure_uvc_device_controls(),
    ]
}

fn view_camera_streams() -> UserJourney {
    UserJourney {
        id: JourneyId("view_camera_streams".into()),
        summary: Grounded::known(
            "Manage video devices and view configured camera streams".into(),
            Provenance::source(VIDEO_MENUS, 132),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(VIDEO_MENUS, 131)),
        services: mcm_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "view_camera_streams",
            "Video Streams page lists detected cameras and their configured streams",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Video Streams page from the sidebar",
                None,
                Provenance::source(VIDEO_MENUS, 128),
                None,
            ),
            operator_step(
                "Load the Video Manager interface listing connected video devices",
                None,
                Provenance::source(VIDEO_MANAGER, 12),
                None,
            ),
            operator_step(
                "Load detected video devices",
                Some(sourced_route(HttpMethod::Get, "/v4l", None, 144)),
                Provenance::source(VIDEO_STORE, 144),
                Some(runtime_outcome(
                    200,
                    Some("\"name\":\"bcm2835-isp\"".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "Load configured video streams",
                Some(sourced_route(HttpMethod::Get, "/streams", None, 168)),
                Provenance::source(VIDEO_STORE, 168),
                Some(runtime_outcome(200, Some("[]".into()), "#running_baseline")),
            ),
            operator_step(
                "View each detected camera card with its name and source path",
                None,
                Provenance::source(VIDEO_DEVICE, 32),
                None,
            ),
            operator_step(
                "View stream cards showing name, encoding, endpoints, source, and status",
                None,
                Provenance::source(VIDEO_STREAM, 7),
                None,
            ),
            operator_step(
                "BlueOS automatically detects H264-encoded video streams on startup",
                None,
                Provenance::doc(ADV, 766),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn configure_camera_stream() -> UserJourney {
    UserJourney {
        id: JourneyId("configure_camera_stream".into()),
        summary: Grounded::known(
            "Manually add and configure a new video stream (encoding, resolution, endpoint type)".into(),
            Provenance::doc(ADV, 780),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 780)),
        services: mcm_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "configure_camera_stream",
            "stream creation dialog submits encoding, resolution, framerate, and endpoints via POST /streams",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::HardwarePresent("H264-capable camera connected to the onboard computer".into()),
            Provenance::doc(GETTING_STARTED, 121),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Video Streams page from the sidebar",
                None,
                Provenance::source(VIDEO_MENUS, 128),
                None,
            ),
            operator_step(
                "Click Add stream on a video device card",
                None,
                Provenance::source(VIDEO_DEVICE, 53),
                None,
            ),
            operator_step(
                "Enter a stream nickname",
                None,
                Provenance::source(VIDEO_STREAM_CREATION_DIALOG, 20),
                None,
            ),
            operator_step(
                "Select the video encoding",
                None,
                Provenance::source(VIDEO_STREAM_CREATION_DIALOG, 27),
                None,
            ),
            operator_step(
                "Select the video resolution",
                None,
                Provenance::source(VIDEO_STREAM_CREATION_DIALOG, 35),
                None,
            ),
            operator_step(
                "Select the framerate",
                None,
                Provenance::source(VIDEO_STREAM_CREATION_DIALOG, 42),
                None,
            ),
            operator_step(
                "Choose the stream endpoint type and address (UDP or RTSP)",
                None,
                Provenance::doc(ADV, 781),
                None,
            ),
            operator_step(
                "Add additional endpoints of the same type with the blue + button",
                None,
                Provenance::doc(ADV, 784),
                None,
            ),
            operator_step(
                "Click Create to add the configured stream",
                Some(sourced_route(HttpMethod::Post, "/streams", None, 120)),
                Provenance::source(VIDEO_STREAM_CREATION_DIALOG, 164),
                Some(pending_outcome(
                    "POST /streams stream creation requires runtime capture with cameras attached (mutating; not exercised)",
                )),
            ),
        ]),
        chains_from: Some(JourneyId("view_camera_streams".into())),
    }
}

fn remove_camera_stream() -> UserJourney {
    UserJourney {
        id: JourneyId("remove_camera_stream".into()),
        summary: Grounded::known(
            "Remove a configured video stream from a camera device".into(),
            Provenance::source(VIDEO_STREAM, 120),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(VIDEO_MENUS, 131)),
        services: mcm_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "remove_camera_stream",
            "stream card remove button deletes the stream via DELETE /delete_stream",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("At least one configured stream is listed on a device card".into()),
            Provenance::source(VIDEO_DEVICE, 82),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Video Streams page from the sidebar",
                None,
                Provenance::source(VIDEO_MENUS, 128),
                None,
            ),
            operator_step(
                "View the configured stream to remove on the device card",
                None,
                Provenance::source(VIDEO_DEVICE, 88),
                None,
            ),
            operator_step(
                "Click the Remove stream button on the stream card",
                None,
                Provenance::source(VIDEO_STREAM, 120),
                None,
            ),
            operator_step(
                "Delete the selected stream",
                Some(sourced_route(HttpMethod::Delete, "/delete_stream", None, 102)),
                Provenance::source(VIDEO_STORE, 102),
                Some(pending_outcome(
                    "DELETE /delete_stream requires runtime capture with a configured stream (mutating; not exercised)",
                )),
            ),
        ]),
        chains_from: Some(JourneyId("view_camera_streams".into())),
    }
}

fn configure_uvc_device_controls() -> UserJourney {
    UserJourney {
        id: JourneyId("configure_uvc_device_controls".into()),
        summary: Grounded::known(
            "Configure UVC camera settings such as brightness and exposure via Device Controls".into(),
            Provenance::doc(ADV, 803),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 803)),
        services: mcm_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "configure_uvc_device_controls",
            "Device Controls dialog adjusts UVC sliders, menus, and booleans via POST /v4l",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::HardwarePresent(
                "UVC camera with exposed controls connected to the onboard computer".into(),
            ),
            Provenance::doc(ADV, 803),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Video Streams page from the sidebar",
                None,
                Provenance::source(VIDEO_MENUS, 128),
                None,
            ),
            operator_step(
                "Click Device Controls on a camera card",
                None,
                Provenance::source(VIDEO_DEVICE, 45),
                None,
            ),
            operator_step(
                "Adjust slider, menu, or boolean camera controls",
                None,
                Provenance::source(VIDEO_CONTROLS_DIALOG, 24),
                None,
            ),
            operator_step(
                "Submit updated control values to the camera manager",
                Some(sourced_route_in(
                    VIDEO_CONTROLS_DIALOG,
                    HttpMethod::Post,
                    "/v4l",
                    None,
                    189,
                )),
                Provenance::source(VIDEO_CONTROLS_DIALOG, 189),
                Some(pending_outcome(
                    "POST /v4l control update requires runtime capture with a UVC camera attached (mutating; not exercised)",
                )),
            ),
        ]),
        chains_from: Some(JourneyId("view_camera_streams".into())),
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn mcm_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId("mavlink-camera-manager".into()),
        Provenance::doc(ADV, 764),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId("mavlink-camera-manager".into()),
        method,
        path: path.into(),
        version: version.map(str::to_string),
    }
}

fn sourced_route(
    method: HttpMethod,
    path: &str,
    version: Option<&str>,
    line: u32,
) -> Grounded<RouteRef> {
    sourced_route_in(VIDEO_STORE, method, path, version, line)
}

fn sourced_route_in(
    file: &str,
    method: HttpMethod,
    path: &str,
    version: Option<&str>,
    line: u32,
) -> Grounded<RouteRef> {
    Grounded::known(route(method, path, version), Provenance::source(file, line))
}

fn operator_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description: description.into(),
            route,
            outcome,
        },
        provenance,
    )
}

fn pending_outcome(reason: &str) -> Grounded<StepOutcome> {
    Grounded::unknown(reason)
}

fn runtime_outcome(status: u16, body: Option<String>, key: &str) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            transition: None,
        },
        Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV),
    )
}
