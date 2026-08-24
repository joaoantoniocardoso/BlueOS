use crate::journey_presence::{
    PRESENCE_BROWSE_VIDEO_RECORDINGS, PRESENCE_DELETE_VIDEO_RECORDING,
    PRESENCE_DOWNLOAD_VIDEO_RECORDING,
};
use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, BodyKind, DataAssumption, HttpMethod, JourneyStep, Precondition, RouteRef,
    StepOutcome, UseCase, Visibility,
};

const RECORDER_MAIN: &str = "core/services/recorder_extractor/main.py";
const RECORDER_MENUS: &str = "core/frontend/src/menus.ts";
const RECORDER_STORE: &str = "core/frontend/src/store/records.ts";
const RECORDS_VIEW: &str = "core/frontend/src/views/RecordsView.vue";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const JOURNEYS: &[UseCase] = &[
    BROWSE_VIDEO_RECORDINGS,
    DOWNLOAD_VIDEO_RECORDING,
    DELETE_VIDEO_RECORDING,
];

const BROWSE_VIDEO_RECORDINGS: UseCase =
    UseCase {
        id: JourneyId::BrowseVideoRecordings,
        summary: Grounded::known(
            "Browse, preview, and download recorded MP4 sessions",
            Provenance::source(RECORDER_MENUS, 139, "text: 'Manage your video devices and video streams.',"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(RECORDER_MENUS, 138, "advanced: false,")),
        services: RECORDER_EXTRACTOR_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::BrowseVideoRecordings,
            "Records page lists MP4 recordings with thumbnails and MCAP extraction processing status",
        )]),
        preconditions: GroundedSet::known(&[]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Records page from the sidebar",
                None,
                Provenance::source(RECORDER_MENUS, 135, "title: 'Video Streams',"),
                None,
            ),
            operator_step(
                "Load the list of available MP4 recordings",
                Some(sourced_route(HttpMethod::Get, "/recorder/files", Some("v1.0"), 340, "@recorder_router.get(")),
                Provenance::source(RECORDER_STORE, 47, "url: `${this.API_URL}/files`,"),
                Some(runtime_outcome(
                    200,
                    Some("[] (empty; no MP4 recordings present)"),
                    "runtime-captures/recorder_extractor__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Load MCAP extraction processing status",
                Some(sourced_route(HttpMethod::Get, "/recorder/status", Some("v1.0"), 370, "@recorder_router.get(")),
                Provenance::source(RECORDER_STORE, 84, "url: `${this.API_URL}/status`,"),
                Some(runtime_outcome(
                    200,
                    Some("\"processing\": [] (empty; no MCAP extraction active)"),
                    "runtime-captures/recorder_extractor__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "View processing cards for recordings still being extracted from MCAP",
                None,
                Provenance::source(RECORDS_VIEW, 39, "Extracting video..."),
                None,
            ),
            operator_step(
                "View recording cards showing name, size, modified date, and thumbnail",
                None,
                Provenance::source(RECORDS_VIEW, 111, "<div class=\"text-truncate\" :title=\"file.name\">"),
                None,
            ),
            operator_step(
                "Load a JPEG thumbnail for each recording card",
                Some(sourced_route(HttpMethod::Get, "/recorder/files/{filename}/thumbnail", Some("v1.0"), 383, "@recorder_ro")),
                Provenance::source(RECORDS_VIEW, 279, "return this.brokenThumbnails[file.path] ? '' : file.thumbnai"),
                Some(pending_outcome(
                    "GET /files/{filename}/thumbnail requires runtime capture with an existing MP4 recording (none present on capture host)",
                )),
            ),
            service_step(
                "Periodically extract MP4 files from MCAP recordings in the background",
                None,
                Provenance::source(RECORDER_MAIN, 261, "async def extract_mcap_recordings() -> None:"),
                None,
            ),
        ]),
        availability: PRESENCE_BROWSE_VIDEO_RECORDINGS,
        blast_radius: Grounded::known(
            BlastRadius::Safe,
            Provenance::asserted(
                "GET /recorder/files and /recorder/status only list recordings and extraction progress",
            ),
        ),
        chains_from: None,
    };

const DOWNLOAD_VIDEO_RECORDING: UseCase =
    UseCase {
        id: JourneyId::DownloadVideoRecording,
        summary: Grounded::known(
            "Download or stream an MP4 recording from the Records gallery",
            Provenance::source(RECORDER_MENUS, 139, "text: 'Manage your video devices and video streams.',"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(RECORDER_MENUS, 138, "advanced: false,")),
        services: RECORDER_EXTRACTOR_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::DownloadVideoRecording,
            "download button or in-dialog player streams the MP4 via GET /files/{filename}",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Data(DataAssumption::RecordingListed),
            Provenance::source(RECORDS_VIEW, 57, "v-for=\"file in recordings\""),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Records page from the sidebar",
                None,
                Provenance::source(RECORDER_MENUS, 135, "title: 'Video Streams',"),
                None,
            ),
            operator_step(
                "Load the list of available MP4 recordings",
                Some(sourced_route(HttpMethod::Get, "/recorder/files", Some("v1.0"), 340, "@recorder_router.get(")),
                Provenance::source(RECORDER_STORE, 47, "url: `${this.API_URL}/files`,"),
                Some(runtime_outcome(
                    200,
                    Some("[] (empty; no MP4 recordings present)"),
                    "runtime-captures/recorder_extractor__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Click the download button on a recording card",
                Some(sourced_route(HttpMethod::Get, "/recorder/files/{filename}", Some("v1.0"), 413, "@recorder_ro")),
                Provenance::source(RECORDS_VIEW, 136, ":href=\"file.download_url\""),
                Some(pending_outcome(
                    "GET /files/{filename} download requires runtime capture with an existing MP4 recording (none present on capture host)",
                )),
            ),
            operator_step(
                "Open the playback dialog and stream the recording",
                Some(sourced_route(HttpMethod::Get, "/recorder/files/{filename}", Some("v1.0"), 413, "@recorder_ro")),
                Provenance::source(RECORDS_VIEW, 173, ":src=\"activeRecord.stream_url\""),
                Some(pending_outcome(
                    "GET /files/{filename} stream playback requires runtime capture with an existing MP4 recording (none present on capture host)",
                )),
            ),
        ]),
        availability: PRESENCE_DOWNLOAD_VIDEO_RECORDING,
        blast_radius: Grounded::known(
            BlastRadius::Safe,
            Provenance::asserted(
                "GET /recorder/files/{filename} streams or downloads without deleting or rewriting recordings",
            ),
        ),
        chains_from: None,
    };

const DELETE_VIDEO_RECORDING: UseCase = UseCase {
    id: JourneyId::DeleteVideoRecording,
    summary: Grounded::known(
        "Delete a recording",
        Provenance::source(RECORDER_MAIN, 397, "summary=\"Delete a recording.\","),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::source(RECORDER_MENUS, 138, "advanced: false,"),
    ),
    services: RECORDER_EXTRACTOR_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DeleteVideoRecording,
        "recording card delete button removes the MP4 file via DELETE /files/{filename}",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataAssumption::RecordingListed),
        Provenance::source(RECORDS_VIEW, 57, "v-for=\"file in recordings\""),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Records page from the sidebar",
            None,
            Provenance::source(RECORDER_MENUS, 135, "title: 'Video Streams',"),
            None,
        ),
        operator_step(
            "Load the list of available MP4 recordings",
            Some(sourced_route(
                HttpMethod::Get,
                "/recorder/files",
                Some("v1.0"),
                340,
                "@recorder_router.get(",
            )),
            Provenance::source(RECORDER_STORE, 47, "url: `${this.API_URL}/files`,"),
            Some(runtime_outcome(
                200,
                Some("[] (empty; no MP4 recordings present)"),
                "runtime-captures/recorder_extractor__pi4_navigator_master.json#running_baseline",
            )),
        ),
        operator_step(
            "Click the delete button on a recording card",
            Some(sourced_route(
                HttpMethod::Delete,
                "/recorder/files/{filename}",
                Some("v1.0"),
                395,
                "@recorder_router.delete(",
            )),
            Provenance::source(RECORDS_VIEW, 126, "@click=\"deleteRecording(file)\""),
            Some(source_outcome(204, 395, "@recorder_router.delete(")),
        ),
    ]),
    availability: PRESENCE_DELETE_VIDEO_RECORDING,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "DELETE /recorder/files/{filename} permanently removes the MP4 with no product undo",
        ),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const RECORDER_EXTRACTOR_SERVICES: GroundedSet<ServiceId> =
    GroundedSet::known(&[GroundedItem::new(
        ServiceId::RecorderExtractor,
        Provenance::source(RECORDER_MAIN, 26, "SERVICE_NAME = \"recorder-extractor\""),
    )]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::RecorderExtractor,
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
    Grounded::known(
        route(method, path, version),
        Provenance::source(RECORDER_MAIN, line, anchor),
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

const fn service_step(
    description: &'static str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Service(ServiceId::RecorderExtractor),
            description,
            route,
            outcome,
        },
        provenance,
    )
}

const fn runtime_body_kind(status: u16, body: Option<&'static str>) -> BodyKind {
    match body {
        Some(_) => BodyKind::Payload,
        None if status == 204 => BodyKind::Empty,
        None => BodyKind::Unknown,
    }
}

const fn pending_outcome(reason: &'static str) -> Grounded<StepOutcome> {
    Grounded::unknown(reason)
}

const fn source_outcome(status: u16, line: u32, anchor: &'static str) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind: BodyKind::Unknown,
            transition: None,
        },
        Provenance::source(RECORDER_MAIN, line, anchor),
    )
}

const fn runtime_outcome(
    status: u16,
    body: Option<&'static str>,
    key: &'static str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            body_kind: runtime_body_kind(status, body),
            transition: None,
        },
        Provenance::runtime(key, RUNTIME_ENV),
    )
}
