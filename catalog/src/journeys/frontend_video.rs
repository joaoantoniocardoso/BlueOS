use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, HardwareAssumption, JourneyStep, Precondition, StepOutcome, UseCase,
    Visibility,
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

pub const JOURNEYS: &[UseCase] = &[CONFIGURE_VIDEO_STREAM];

const CONFIGURE_VIDEO_STREAM: UseCase = UseCase {
    id: JourneyId::ConfigureVideoStream,
    summary: Grounded::known(
        "Operator adds or reconfigures a camera video stream on the Video Streams page; the browser builds and validates the endpoint and drives create/replace against mavlink-camera-manager",
        Provenance::doc(ADV, 763, "### Video Streams"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::source(MENUS, 128, "title: 'Vehicle Setup',")),
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
        Precondition::Hardware(HardwareAssumption::UsbCamera),
        Provenance::doc(ADV, 766, "- BlueOS automatically detects H264-encoded video streams"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Video Streams page from the sidebar",
            Provenance::source(MENUS, 128, "title: 'Vehicle Setup',"),
        ),
        frontend_step(
            "VideoManager filters the backend device list to displayable cameras before rendering",
            Provenance::source(MANAGER, 123, "video_devices(): Device[] {"),
            None,
        ),
        operator_step(
            "Open the stream creation dialog on a camera and confirm the endpoint",
            Provenance::source(CREATION, 164, "@click=\"createStream\""),
        ),
        frontend_step(
            "createStream validates the endpoint and emits the stream configuration",
            Provenance::source(CREATION, 405, "createStream(): boolean | string {"),
            None,
        ),
        frontend_step(
            "Store creates the stream via POST /mavlink-camera-manager/streams",
            Provenance::source(VIDEO_STORE, 120, "url: `${this.API_URL}/streams`,"),
            Some(mutation_outcome()),
        ),
        frontend_step(
            "editStream replaces an existing stream by deleting then re-creating it",
            Provenance::source(STREAM, 302, "async editStream(edited_stream: CreatedStream): Promise<void"),
            None,
        ),
        frontend_step(
            "VideoDiagnosticHelper flags whether the endpoint is reachable from the client or vehicle IP",
            Provenance::source(DIAG, 52, "route.startsWith(`rtsp://${this.vehicle_ip_address}`) || rou"),
            None,
        ),
        frontend_step(
            "VideoThumbnail previews the stream in snapshot or continuous (1s) mode",
            Provenance::source(THUMB, 154, "&& (this.continuous_mode || this.snapshot_in_progress)"),
            None,
        ),
    ]),
    availability: PRESENCE_CONFIGURE_VIDEO_STREAM,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "Frontend create/replace flow mutates mavlink-camera-manager stream records removable via delete stream",
        ),
    ),
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
        Provenance::source(VIDEO_STORE, 120, "url: `${this.API_URL}/streams`,"),
    ),
    GroundedItem::new(
        ServiceId::Commander,
        Provenance::source(
            COMMANDER_STORE,
            115,
            "url: `${this.API_URL}/raspi_config/camera_legacy`,",
        ),
    ),
    GroundedItem::new(
        ServiceId::Beacon,
        Provenance::source(DIAG, 29, "import beacon from '@/store/beacon'"),
    ),
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
