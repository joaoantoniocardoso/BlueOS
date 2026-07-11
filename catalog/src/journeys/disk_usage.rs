use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const DISK_MAIN: &str = "core/services/disk_usage/main.py";
const DISK_MENUS: &str = "core/frontend/src/menus.ts";
const DISK_VIEW: &str = "core/frontend/src/views/Disk.vue";
const RUNTIME_CAPTURE: &str = "runtime-captures/disk_usage__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn journeys() -> Vec<UserJourney> {
    vec![
        inspect_disk_usage(),
        free_disk_space(),
        run_single_disk_speed_test(),
        run_multi_size_disk_speed_test(),
    ]
}

fn inspect_disk_usage() -> UserJourney {
    UserJourney {
        id: JourneyId::InspectDiskUsage,
        summary: Grounded::known(
            "Get the disk usage tree for a given path".into(),
            Provenance::source(DISK_MAIN, 251),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
        services: disk_usage_services(),
        capability_refs: GroundedSet::known(vec![
            cap(
                CapabilityId::InspectDiskUsage,
                "Disk Usage tab loads a du-backed usage tree for the selected path",
            ),
            cap(
                CapabilityId::NavigateDiskUsage,
                "operator drills into subdirectories by re-fetching usage for a new path",
            ),
        ]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Disk page from the sidebar",
                None,
                Provenance::source(DISK_MENUS, 49),
                None,
            ),
            operator_step(
                "Load the disk usage tree for the current path",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/disk/usage",
                    Some("v1.0"),
                    248,
                )),
                Provenance::source(DISK_MAIN, 251),
                Some(runtime_outcome(
                    200,
                    Some("\"root\": {\"name\": \"/\"".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "Open a subdirectory to inspect how storage is distributed beneath it",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/disk/usage",
                    Some("v1.0"),
                    248,
                )),
                Provenance::source(DISK_MAIN, 255),
                Some(runtime_outcome(
                    200,
                    Some("\"root\": {\"name\": \"/\"".into()),
                    "#running_baseline",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn free_disk_space() -> UserJourney {
    UserJourney {
        id: JourneyId::FreeDiskSpace,
        summary: Grounded::known(
            "Delete files/folders from the Disk tool".into(),
            Provenance::source(DISK_MENUS, 51),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
        services: disk_usage_services(),
        capability_refs: GroundedSet::known(vec![
            cap(
                CapabilityId::InspectDiskUsage,
                "operator browses the usage tree to choose deletion targets",
            ),
            cap(
                CapabilityId::DeleteDiskPaths,
                "selected files or folders are removed recursively via the disk usage API",
            ),
        ]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other(
                "Onboard storage is low or the high disk usage warning is shown".into(),
            ),
            Provenance::doc(ADV, 179),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Disk page from the sidebar",
                None,
                Provenance::source(DISK_MENUS, 49),
                None,
            ),
            operator_step(
                "Browse the disk usage tree to identify files or folders to remove",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/disk/usage",
                    Some("v1.0"),
                    248,
                )),
                Provenance::source(DISK_MAIN, 251),
                Some(runtime_outcome(
                    200,
                    Some("\"root\": {\"name\": \"/\"".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "Select one or more paths to delete",
                None,
                Provenance::source(DISK_VIEW, 109),
                None,
            ),
            operator_step(
                "Delete the selected paths",
                Some(sourced_route(
                    HttpMethod::Delete,
                    "/disk/paths/{target_path}",
                    Some("v1.0"),
                    268,
                )),
                Provenance::source(DISK_MAIN, 270),
                Some(Grounded::unknown("destructive; not exercised in capture")),
            ),
            operator_step(
                "Refresh the disk usage tree to confirm reclaimed space",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/disk/usage",
                    Some("v1.0"),
                    248,
                )),
                Provenance::source(DISK_MAIN, 251),
                Some(runtime_outcome(
                    200,
                    Some("\"root\": {\"name\": \"/\"".into()),
                    "#running_baseline",
                )),
            ),
        ]),
        chains_from: Some(JourneyId::InspectDiskUsage),
    }
}

fn run_single_disk_speed_test() -> UserJourney {
    UserJourney {
        id: JourneyId::RunSingleDiskSpeedTest,
        summary: Grounded::known(
            "Run a single-size disk read/write speed benchmark using the disktest binary".into(),
            Provenance::source(DISK_MAIN, 413),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
        services: disk_usage_services(),
        capability_refs: GroundedSet::known(vec![cap(
            CapabilityId::RunDiskSpeedTest,
            "GET /disk/speed runs one disktest write-and-verify pass at the requested size",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("disktest binary is available on PATH".into()),
            Provenance::source(DISK_MAIN, 323),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Switch to the Speed Test tab",
                None,
                Provenance::source(DISK_VIEW, 15),
                None,
            ),
            operator_step(
                "Start a disk speed test at the chosen test size",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/disk/speed",
                    Some("v1.0"),
                    410,
                )),
                Provenance::source(DISK_MAIN, 413),
                Some(runtime_outcome(
                    200,
                    Some("\"success\": true".into()),
                    "#transitions",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn run_multi_size_disk_speed_test() -> UserJourney {
    UserJourney {
        id: JourneyId::RunMultiSizeDiskSpeedTest,
        summary: Grounded::known(
            "Run a progressive multi-size disk speed benchmark with streaming results".into(),
            Provenance::source(DISK_MAIN, 446),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
        services: disk_usage_services(),
        capability_refs: GroundedSet::known(vec![cap(CapabilityId::RunMultiSizeDiskSpeedTest,
            "GET /disk/speed/stream yields NDJSON points for each benchmark size",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("disktest binary is available on PATH".into()),
            Provenance::source(DISK_MAIN, 323),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Switch to the Speed Test tab",
                None,
                Provenance::source(DISK_VIEW, 15),
                None,
            ),
            operator_step(
                "Start the multi-size disk speed benchmark",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/disk/speed/stream",
                    Some("v1.0"),
                    444,
                )),
                Provenance::source(DISK_MAIN, 446),
                Some(Grounded::unknown(
                    "GET /disk/speed/stream not exercised in capture (multi-size benchmark is long-running)",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: CapabilityId, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

fn disk_usage_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::DiskUsage,
        Provenance::source(DISK_MAIN, 26),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId::DiskUsage,
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
        Provenance::source(DISK_MAIN, line),
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
