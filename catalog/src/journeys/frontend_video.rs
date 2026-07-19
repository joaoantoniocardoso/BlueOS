use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HardwareRequirement, JourneyStep, Precondition, StepOutcome, UserJourney, Visibility,
};
use crate::journey_presence::PRESENCE_CONFIGURE_VIDEO_STREAM;
use crate::page::PageId;
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const MENUS: &str = "core/frontend/src/menus.ts";
const MANAGER: &str = "core/frontend/src/components/video-manager/VideoManager.vue";
const CREATION: &str = "core/frontend/src/components/video-manager/VideoStreamCreationDialog.vue";
const STREAM: &str = "core/frontend/src/components/video-manager/VideoStream.vue";
const DIAG: &str = "core/frontend/src/components/video-manager/VideoDiagnosticHelper.vue";
const THUMB: &str = "core/frontend/src/components/video-manager/VideoThumbnail.vue";
const VIDEO_STORE: &str = "core/frontend/src/store/video.ts";
const COMMANDER_STORE: &str = "core/frontend/src/store/commander.ts";

pub const JOURNEYS: &[UserJourney] = &[CONFIGURE_VIDEO_STREAM];

const CONFIGURE_VIDEO_STREAM: UserJourney = UserJourney {
    id: JourneyId::ConfigureVideoStream,
    summary: Grounded::known(
        "Operator adds or reconfigures a camera video stream on the Video Streams page; the browser builds and validates the endpoint and drives create/replace against mavlink-camera-manager",
        Provenance::doc(ADV, 763),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::source(MENUS, 128)),
    services: SERVICES,
    capability_refs: GroundedSet::known(&[
        cap(
            CapabilityId::FilterDisplayableDevices,
            "VideoManager.vue video_devices computed hides unsupported encodes and secondary streams and sorts devices client-side",
        ),
        cap(
            CapabilityId::ConfigureStreamEndpoints,
            "VideoStreamCreationDialog.vue validates UDP/RTSP endpoints and builds default addresses from beacon IPs",
        ),
        cap(
            CapabilityId::ReplaceStreamConfiguration,
            "VideoStream.vue editStream deletes the existing stream then creates a new one (no in-place update route)",
        ),
        cap(
            CapabilityId::DiagnoseStreamAccessibility,
            "VideoDiagnosticHelper.vue derives whether stream endpoints target the client or vehicle IP from cached streams and beacon addresses",
        ),
        cap(
            CapabilityId::ManageThumbnailPreview,
            "VideoThumbnail.vue orchestrates snapshot vs continuous preview modes with debounce and cooldown",
        ),
    ]),
    preconditions: GroundedSet::known(&[precond(
        Precondition::Hardware(HardwareRequirement::UsbCamera),
        Provenance::doc(ADV, 766),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Video Streams page from the sidebar",
            Provenance::source(MENUS, 128),
        ),
        frontend_step(
            "VideoManager filters the backend device list to displayable cameras before rendering",
            Provenance::source(MANAGER, 123),
            None,
        ),
        operator_step(
            "Open the stream creation dialog on a camera and confirm the endpoint",
            Provenance::source(CREATION, 164),
        ),
        frontend_step(
            "createStream validates the endpoint and emits the stream configuration",
            Provenance::source(CREATION, 405),
            None,
        ),
        frontend_step(
            "Store creates the stream via POST /mavlink-camera-manager/streams",
            Provenance::source(VIDEO_STORE, 120),
            Some(mutation_outcome()),
        ),
        frontend_step(
            "editStream replaces an existing stream by deleting then re-creating it",
            Provenance::source(STREAM, 302),
            None,
        ),
        frontend_step(
            "VideoDiagnosticHelper flags whether the endpoint is reachable from the client or vehicle IP",
            Provenance::source(DIAG, 52),
            None,
        ),
        frontend_step(
            "VideoThumbnail previews the stream in snapshot or continuous (1s) mode",
            Provenance::source(THUMB, 154),
            None,
        ),
    ]),
    availability: PRESENCE_CONFIGURE_VIDEO_STREAM,
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const fn precond(value: Precondition, provenance: Provenance) -> GroundedItem<Precondition> {
    GroundedItem::new(value, provenance)
}

const SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[
    GroundedItem::new(
        ServiceId::MavlinkCameraManager,
        Provenance::source(VIDEO_STORE, 120),
    ),
    GroundedItem::new(
        ServiceId::Commander,
        Provenance::source(COMMANDER_STORE, 115),
    ),
    GroundedItem::new(ServiceId::Beacon, Provenance::source(DIAG, 29)),
]);

const fn operator_step(
    description: &'static str,
    provenance: Provenance,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description,
            route: None,
            outcome: None,
        },
        provenance,
    )
}

const fn frontend_step(
    description: &'static str,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Frontend(PageId::VideoManager),
            description,
            route: None,
            outcome,
        },
        provenance,
    )
}

const fn mutation_outcome() -> Grounded<StepOutcome> {
    Grounded::unknown(
        "stream create/replace result requires runtime capture during a live stream configuration on the Pi (mutating; not performed)",
    )
}
