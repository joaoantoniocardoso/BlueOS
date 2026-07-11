use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const RECORDER_MAIN: &str = "core/services/recorder_extractor/main.py";
const RECORDER_MENUS: &str = "core/frontend/src/menus.ts";
const RECORDER_STORE: &str = "core/frontend/src/store/records.ts";
const RECORDS_VIEW: &str = "core/frontend/src/views/RecordsView.vue";
const RUNTIME_CAPTURE: &str = "runtime-captures/recorder_extractor__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn journeys() -> Vec<UserJourney> {
    vec![
        browse_video_recordings(),
        download_video_recording(),
        delete_video_recording(),
    ]
}

fn browse_video_recordings() -> UserJourney {
    UserJourney {
        id: JourneyId::BrowseVideoRecordings,
        summary: Grounded::known(
            "Browse, preview, and download recorded MP4 sessions".into(),
            Provenance::source(RECORDER_MENUS, 139),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(RECORDER_MENUS, 138)),
        services: recorder_extractor_services(),
        capability_refs: GroundedSet::known(vec![cap(CapabilityId::BrowseVideoRecordings,
            "Records page lists MP4 recordings with thumbnails and MCAP extraction processing status",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Records page from the sidebar",
                None,
                Provenance::source(RECORDER_MENUS, 135),
                None,
            ),
            operator_step(
                "Load the list of available MP4 recordings",
                Some(sourced_route(HttpMethod::Get, "/files", Some("v1.0"), 340)),
                Provenance::source(RECORDER_STORE, 47),
                Some(runtime_outcome(
                    200,
                    Some("[] (empty; no MP4 recordings present)".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "Load MCAP extraction processing status",
                Some(sourced_route(HttpMethod::Get, "/status", Some("v1.0"), 370)),
                Provenance::source(RECORDER_STORE, 84),
                Some(runtime_outcome(
                    200,
                    Some("\"processing\": [] (empty; no MCAP extraction active)".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "View processing cards for recordings still being extracted from MCAP",
                None,
                Provenance::source(RECORDS_VIEW, 39),
                None,
            ),
            operator_step(
                "View recording cards showing name, size, modified date, and thumbnail",
                None,
                Provenance::source(RECORDS_VIEW, 111),
                None,
            ),
            operator_step(
                "Load a JPEG thumbnail for each recording card",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/files/{filename}/thumbnail",
                    Some("v1.0"),
                    383,
                )),
                Provenance::source(RECORDS_VIEW, 279),
                Some(pending_outcome(
                    "GET /files/{filename}/thumbnail requires runtime capture with an existing MP4 recording (none present on capture host)",
                )),
            ),
            service_step(
                "Periodically extract MP4 files from MCAP recordings in the background",
                None,
                Provenance::source(RECORDER_MAIN, 261),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn download_video_recording() -> UserJourney {
    UserJourney {
        id: JourneyId::DownloadVideoRecording,
        summary: Grounded::known(
            "Download or stream an MP4 recording from the Records gallery".into(),
            Provenance::source(RECORDER_MENUS, 139),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(RECORDER_MENUS, 138)),
        services: recorder_extractor_services(),
        capability_refs: GroundedSet::known(vec![cap(CapabilityId::DownloadVideoRecording,
            "download button or in-dialog player streams the MP4 via GET /files/{filename}",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("At least one MP4 recording is listed on the Records page".into()),
            Provenance::source(RECORDS_VIEW, 57),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Records page from the sidebar",
                None,
                Provenance::source(RECORDER_MENUS, 135),
                None,
            ),
            operator_step(
                "Load the list of available MP4 recordings",
                Some(sourced_route(HttpMethod::Get, "/files", Some("v1.0"), 340)),
                Provenance::source(RECORDER_STORE, 47),
                Some(runtime_outcome(
                    200,
                    Some("[] (empty; no MP4 recordings present)".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "Click the download button on a recording card",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/files/{filename}",
                    Some("v1.0"),
                    413,
                )),
                Provenance::source(RECORDS_VIEW, 136),
                Some(pending_outcome(
                    "GET /files/{filename} download requires runtime capture with an existing MP4 recording (none present on capture host)",
                )),
            ),
            operator_step(
                "Open the playback dialog and stream the recording",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/files/{filename}",
                    Some("v1.0"),
                    413,
                )),
                Provenance::source(RECORDS_VIEW, 173),
                Some(pending_outcome(
                    "GET /files/{filename} stream playback requires runtime capture with an existing MP4 recording (none present on capture host)",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn delete_video_recording() -> UserJourney {
    UserJourney {
        id: JourneyId::DeleteVideoRecording,
        summary: Grounded::known(
            "Delete a recording".into(),
            Provenance::source(RECORDER_MAIN, 397),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(RECORDER_MENUS, 138)),
        services: recorder_extractor_services(),
        capability_refs: GroundedSet::known(vec![cap(CapabilityId::DeleteVideoRecording,
            "recording card delete button removes the MP4 file via DELETE /files/{filename}",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("At least one MP4 recording is listed on the Records page".into()),
            Provenance::source(RECORDS_VIEW, 57),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Records page from the sidebar",
                None,
                Provenance::source(RECORDER_MENUS, 135),
                None,
            ),
            operator_step(
                "Load the list of available MP4 recordings",
                Some(sourced_route(HttpMethod::Get, "/files", Some("v1.0"), 340)),
                Provenance::source(RECORDER_STORE, 47),
                Some(runtime_outcome(
                    200,
                    Some("[] (empty; no MP4 recordings present)".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "Click the delete button on a recording card",
                Some(sourced_route(
                    HttpMethod::Delete,
                    "/files/{filename}",
                    Some("v1.0"),
                    395,
                )),
                Provenance::source(RECORDS_VIEW, 126),
                Some(pending_outcome(
                    "DELETE /files/{filename} permanently removes the MP4 file (destructive; not exercised; no recordings present on capture host)",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: CapabilityId, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

fn recorder_extractor_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::RecorderExtractor,
        Provenance::source(RECORDER_MAIN, 26),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId::RecorderExtractor,
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
    Grounded::known(
        route(method, path, version),
        Provenance::source(RECORDER_MAIN, line),
    )
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

fn service_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Service(ServiceId::RecorderExtractor),
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
