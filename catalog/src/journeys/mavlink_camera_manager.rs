use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, BodyKind, HardwareAssumption, HttpMethod, JourneyStep, Precondition,
    RouteRef, StepOutcome, UseCase, Visibility,
};
use crate::journey_presence::{
    PRESENCE_CONFIGURE_CAMERA_STREAM, PRESENCE_CONFIGURE_UVC_DEVICE_CONTROLS,
    PRESENCE_REMOVE_CAMERA_STREAM, PRESENCE_VIEW_CAMERA_STREAMS,
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
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const BR_VIEW_CAMERA_STREAMS: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Safe,
    Provenance::asserted("GET /v4l and GET /streams only enumerate devices and configured streams"),
);
const BR_CONFIGURE_CAMERA_STREAM: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Reversible,
    Provenance::asserted(
        "POST /streams adds a stream configuration removable via DELETE /delete_stream",
    ),
);
const BR_REMOVE_CAMERA_STREAM: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Reversible,
    Provenance::asserted(
        "DELETE /delete_stream removes one configured stream without affecting the device",
    ),
);
const BR_CONFIGURE_UVC_DEVICE_CONTROLS: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Reversible,
    Provenance::asserted(
        "POST /v4l updates UVC control values adjustable again or reset on the camera",
    ),
);

pub const JOURNEYS: &[UseCase] = &[
    VIEW_CAMERA_STREAMS,
    CONFIGURE_CAMERA_STREAM,
    REMOVE_CAMERA_STREAM,
    CONFIGURE_UVC_DEVICE_CONTROLS,
];

const VIEW_CAMERA_STREAMS: UseCase =
    UseCase {
        id: JourneyId::ViewCameraStreams,
        summary: Grounded::known(
            "Manage video devices and view configured camera streams",
            Provenance::source(VIDEO_MENUS, 132, "text: 'Vehicle and Peripherals setup. Includes sensor calibr"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(VIDEO_MENUS, 131, "advanced: false,")),
        services: MCM_SERVICES,
        capability_refs: GroundedSet::known(&[cap(
            CapabilityId::ViewCameraStreams,
            "Video Streams page lists detected cameras and their configured streams",
        )]),
        preconditions: GroundedSet::known(&[]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Video Streams page from the sidebar",
                None,
                Provenance::source(VIDEO_MENUS, 128, "title: 'Vehicle Setup',"),
                None,
            ),
            operator_step(
                "Load the Video Manager interface listing connected video devices",
                None,
                Provenance::source(VIDEO_MANAGER, 12, "v-for=\"(device, index) in video_devices\""),
                None,
            ),
            operator_step(
                "Load detected video devices",
                Some(sourced_route(HttpMethod::Get, "/v4l", None, 144, "url: `${this.API_URL}/v4l`,")),
                Provenance::source(VIDEO_STORE, 144, "url: `${this.API_URL}/v4l`,"),
                Some(runtime_outcome(
                    200,
                    Some("\"name\":\"bcm2835-isp\""),
                    "runtime-captures/mavlink_camera_manager__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Load configured video streams",
                Some(sourced_route(HttpMethod::Get, "/streams", None, 168, "url: `${this.API_URL}/streams`,")),
                Provenance::source(VIDEO_STORE, 168, "url: `${this.API_URL}/streams`,"),
                Some(runtime_outcome(200, Some("[]"), "runtime-captures/mavlink_camera_manager__pi4_navigator_master.json#running_baseline")),
            ),
            operator_step(
                "View each detected camera card with its name and source path",
                None,
                Provenance::source(VIDEO_DEVICE, 32, "{{ device.name }}"),
                None,
            ),
            operator_step(
                "View stream cards showing name, encoding, endpoints, source, and status",
                None,
                Provenance::source(VIDEO_STREAM, 7, "Stream Name:"),
                None,
            ),
            operator_step(
                "BlueOS automatically detects H264-encoded video streams on startup",
                None,
                Provenance::doc(ADV, 766, "- BlueOS automatically detects H264-encoded video streams"),
                None,
            ),
        ]),
        availability: PRESENCE_VIEW_CAMERA_STREAMS,
        blast_radius: BR_VIEW_CAMERA_STREAMS,
        chains_from: None,
    };

const CONFIGURE_CAMERA_STREAM: UseCase =
    UseCase {
        id: JourneyId::ConfigureCameraStream,
        summary: Grounded::known(
            "Manually add and configure a new video stream (encoding, resolution, endpoint type)",
            Provenance::doc(ADV, 780, "- New streams need to be manually added"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 780, "- New streams need to be manua")),
        services: MCM_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ConfigureCameraStream,
            "stream creation dialog submits encoding, resolution, framerate, and endpoints via POST /streams",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Hardware(HardwareAssumption::UsbCamera),
            Provenance::doc(GETTING_STARTED, 121, "BlueOS is capable of configuring and streaming multiple came"),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Video Streams page from the sidebar",
                None,
                Provenance::source(VIDEO_MENUS, 128, "title: 'Vehicle Setup',"),
                None,
            ),
            operator_step(
                "Click Add stream on a video device card",
                None,
                Provenance::source(VIDEO_DEVICE, 53, "Add stream"),
                None,
            ),
            operator_step(
                "Enter a stream nickname",
                None,
                Provenance::source(VIDEO_STREAM_CREATION_DIALOG, 20, "label=\"Stream nickname\""),
                None,
            ),
            operator_step(
                "Select the video encoding",
                None,
                Provenance::source(VIDEO_STREAM_CREATION_DIALOG, 27, "label=\"Encoding\""),
                None,
            ),
            operator_step(
                "Select the video resolution",
                None,
                Provenance::source(VIDEO_STREAM_CREATION_DIALOG, 35, ":label=\"size_selector_label\""),
                None,
            ),
            operator_step(
                "Select the framerate",
                None,
                Provenance::source(VIDEO_STREAM_CREATION_DIALOG, 42, ":label=\"framerate_selector_label\""),
                None,
            ),
            operator_step(
                "Choose the stream endpoint type and address (UDP or RTSP)",
                None,
                Provenance::doc(ADV, 781, "- UDP stream endpoints should be set to `udp://<surface-IP>:"),
                None,
            ),
            operator_step(
                "Add additional endpoints of the same type with the blue + button",
                None,
                Provenance::doc(ADV, 784, "- One video input can have multiple output streams by clicki"),
                None,
            ),
            operator_step(
                "Click Create to add the configured stream",
                Some(sourced_route(HttpMethod::Post, "/streams", None, 120, "url: `${this.API_URL}/streams`,")),
                Provenance::source(VIDEO_STREAM_CREATION_DIALOG, 164, "@click=\"createStream\""),
                Some(pending_outcome(
                    "POST /streams stream creation requires runtime capture with cameras attached (mutating; not exercised)",
                )),
            ),
        ]),
        availability: PRESENCE_CONFIGURE_CAMERA_STREAM,
        blast_radius: BR_CONFIGURE_CAMERA_STREAM,
        chains_from: Some(JourneyId::ViewCameraStreams),
    };

const REMOVE_CAMERA_STREAM: UseCase = UseCase {
    id: JourneyId::RemoveCameraStream,
    summary: Grounded::known(
        "Remove a configured video stream from a camera device",
        Provenance::source(VIDEO_STREAM, 120, "v-tooltip=\"'Remove stream'\""),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::source(VIDEO_MENUS, 131, "advanced: false,"),
    ),
    services: MCM_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RemoveCameraStream,
        "stream card remove button deletes the stream via DELETE /delete_stream",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("At least one configured stream is listed on a device card"),
        Provenance::source(
            VIDEO_DEVICE,
            82,
            "<v-container v-if=\"are_video_streams_available && !updating_",
        ),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Video Streams page from the sidebar",
            None,
            Provenance::source(VIDEO_MENUS, 128, "title: 'Vehicle Setup',"),
            None,
        ),
        operator_step(
            "View the configured stream to remove on the device card",
            None,
            Provenance::source(VIDEO_DEVICE, 88, "<video-stream"),
            None,
        ),
        operator_step(
            "Click the Remove stream button on the stream card",
            None,
            Provenance::source(VIDEO_STREAM, 120, "v-tooltip=\"'Remove stream'\""),
            None,
        ),
        operator_step(
            "Delete the selected stream",
            Some(sourced_route(
                HttpMethod::Delete,
                "/delete_stream",
                None,
                102,
                "url: `${this.API_URL}/delete_stream`,",
            )),
            Provenance::source(VIDEO_STORE, 102, "url: `${this.API_URL}/delete_stream`,"),
            Some(source_outcome(
                200,
                VIDEO_STORE,
                102,
                "url: `${this.API_URL}/delete_stream`,",
            )),
        ),
    ]),
    availability: PRESENCE_REMOVE_CAMERA_STREAM,
    blast_radius: BR_REMOVE_CAMERA_STREAM,
    chains_from: Some(JourneyId::ViewCameraStreams),
};

const CONFIGURE_UVC_DEVICE_CONTROLS: UseCase =
    UseCase {
        id: JourneyId::ConfigureUvcDeviceControls,
        summary: Grounded::known(
            "Configure UVC camera settings such as brightness and exposure via Device Controls",
            Provenance::doc(ADV, 803, "- Camera settings (brightness, exposure, etc) that are expos"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 803, "- Camera settings (brightness,")),
        services: MCM_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ConfigureUvcDeviceControls,
            "Device Controls dialog adjusts UVC sliders, menus, and booleans via POST /v4l",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Hardware(HardwareAssumption::UsbCamera),
            Provenance::doc(ADV, 803, "- Camera settings (brightness, exposure, etc) that are expos"),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Video Streams page from the sidebar",
                None,
                Provenance::source(VIDEO_MENUS, 128, "title: 'Vehicle Setup',"),
                None,
            ),
            operator_step(
                "Click Device Controls on a camera card",
                None,
                Provenance::source(VIDEO_DEVICE, 45, "Device Controls"),
                None,
            ),
            operator_step(
                "Adjust slider, menu, or boolean camera controls",
                None,
                Provenance::source(VIDEO_CONTROLS_DIALOG, 24, "<v-slider"),
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
                    "method: 'post',",
                )),
                Provenance::source(VIDEO_CONTROLS_DIALOG, 189, "method: 'post',"),
                Some(pending_outcome(
                    "POST /v4l control update requires runtime capture with a UVC camera attached (mutating; not exercised)",
                )),
            ),
        ]),
        availability: PRESENCE_CONFIGURE_UVC_DEVICE_CONTROLS,
        blast_radius: BR_CONFIGURE_UVC_DEVICE_CONTROLS,
        chains_from: Some(JourneyId::ViewCameraStreams),
    };

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const MCM_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::MavlinkCameraManager,
    Provenance::doc(
        ADV,
        764,
        "{{ service(service=\"MAVLink Camera Manager\", link=\"https://g",
    ),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::MavlinkCameraManager,
        method,
        path,
        version,
    }
}

const fn sourced_route(
    method: HttpMethod,
    path: &'static str,
    version: Option<&'static str>,
    line: u32,
    anchor: &'static str,
) -> Grounded<RouteRef> {
    sourced_route_in(VIDEO_STORE, method, path, version, line, anchor)
}

const fn sourced_route_in(
    file: &'static str,
    method: HttpMethod,
    path: &'static str,
    version: Option<&'static str>,
    line: u32,
    anchor: &'static str,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(file, line, anchor),
    )
}

const fn operator_step(
    description: &'static str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description,
            route,
            outcome,
        },
        provenance,
    )
}

const fn pending_outcome(reason: &'static str) -> Grounded<StepOutcome> {
    Grounded::unknown(reason)
}

const fn source_outcome(
    status: u16,
    file: &'static str,
    line: u32,
    anchor: &'static str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind: BodyKind::Unknown,
            transition: None,
        },
        Provenance::source(file, line, anchor),
    )
}

const fn runtime_outcome(
    status: u16,
    body: Option<&'static str>,
    key: &'static str,
) -> Grounded<StepOutcome> {
    let body_kind = if body.is_some() {
        BodyKind::Payload
    } else {
        BodyKind::Unknown
    };
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            body_kind,
            transition: None,
        },
        Provenance::runtime(key, RUNTIME_ENV),
    )
}
