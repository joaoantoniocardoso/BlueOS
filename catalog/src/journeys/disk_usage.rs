use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::journey_presence::{
    PRESENCE_FREE_DISK_SPACE, PRESENCE_INSPECT_DISK_USAGE, PRESENCE_RUN_MULTI_SIZE_DISK_SPEED_TEST,
    PRESENCE_RUN_SINGLE_DISK_SPEED_TEST,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const DISK_MAIN: &str = "core/services/disk_usage/main.py";
const DISK_MENUS: &str = "core/frontend/src/menus.ts";
const DISK_VIEW: &str = "core/frontend/src/views/Disk.vue";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const JOURNEYS: &[UserJourney] = &[
    INSPECT_DISK_USAGE,
    FREE_DISK_SPACE,
    RUN_SINGLE_DISK_SPEED_TEST,
    RUN_MULTI_SIZE_DISK_SPEED_TEST,
];

const INSPECT_DISK_USAGE: UserJourney = UserJourney {
    id: JourneyId::InspectDiskUsage,
    summary: Grounded::known(
        "Get the disk usage tree for a given path",
        Provenance::source(DISK_MAIN, 251),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
    services: DISK_USAGE_SERVICES,
    capability_refs: GroundedSet::known(&[
        cap(
            CapabilityId::InspectDiskUsage,
            "Disk Usage tab loads a du-backed usage tree for the selected path",
        ),
        cap(
            CapabilityId::NavigateDiskUsage,
            "operator drills into subdirectories by re-fetching usage for a new path",
        ),
    ]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
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
                Some("\"root\": {\"name\": \"/\""),
                "runtime-captures/disk_usage__pi4_navigator_master.json#running_baseline",
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
                Some("\"root\": {\"name\": \"/\""),
                "runtime-captures/disk_usage__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_INSPECT_DISK_USAGE,
    chains_from: None,
};

const FREE_DISK_SPACE: UserJourney = UserJourney {
    id: JourneyId::FreeDiskSpace,
    summary: Grounded::known(
        "Delete files/folders from the Disk tool",
        Provenance::source(DISK_MENUS, 51),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
    services: DISK_USAGE_SERVICES,
    capability_refs: GroundedSet::known(&[
        cap(
            CapabilityId::InspectDiskUsage,
            "operator browses the usage tree to choose deletion targets",
        ),
        cap(
            CapabilityId::DeleteDiskPaths,
            "selected files or folders are removed recursively via the disk usage API",
        ),
    ]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("Onboard storage is low or the high disk usage warning is shown"),
        Provenance::doc(ADV, 179),
    )]),
    steps: GroundedSet::known(&[
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
                Some("\"root\": {\"name\": \"/\""),
                "runtime-captures/disk_usage__pi4_navigator_master.json#running_baseline",
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
            Some(source_outcome(204, 270)),
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
                Some("\"root\": {\"name\": \"/\""),
                "runtime-captures/disk_usage__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_FREE_DISK_SPACE,
    chains_from: Some(JourneyId::InspectDiskUsage),
};

const RUN_SINGLE_DISK_SPEED_TEST: UserJourney = UserJourney {
    id: JourneyId::RunSingleDiskSpeedTest,
    summary: Grounded::known(
        "Run a single-size disk read/write speed benchmark using the disktest binary",
        Provenance::source(DISK_MAIN, 413),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
    services: DISK_USAGE_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RunDiskSpeedTest,
        "GET /disk/speed runs one disktest write-and-verify pass at the requested size",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("disktest binary is available on PATH"),
        Provenance::source(DISK_MAIN, 323),
    )]),
    steps: GroundedSet::known(&[
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
                Some("\"success\": true"),
                "runtime-captures/disk_usage__pi4_navigator_master.json#transitions",
            )),
        ),
    ]),
    availability: PRESENCE_RUN_SINGLE_DISK_SPEED_TEST,
    chains_from: None,
};

const RUN_MULTI_SIZE_DISK_SPEED_TEST: UserJourney = UserJourney {
    id: JourneyId::RunMultiSizeDiskSpeedTest,
    summary: Grounded::known(
        "Run a progressive multi-size disk speed benchmark with streaming results",
        Provenance::source(DISK_MAIN, 446),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
    services: DISK_USAGE_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RunMultiSizeDiskSpeedTest,
        "GET /disk/speed/stream yields NDJSON points for each benchmark size",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("disktest binary is available on PATH"),
        Provenance::source(DISK_MAIN, 323),
    )]),
    steps: GroundedSet::known(&[
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
            Some(source_outcome(200, 448)),
        ),
    ]),
    availability: PRESENCE_RUN_MULTI_SIZE_DISK_SPEED_TEST,
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const DISK_USAGE_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::DiskUsage,
    Provenance::source(DISK_MAIN, 26),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::DiskUsage,
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
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(DISK_MAIN, line),
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

const fn source_outcome(status: u16, line: u32) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            transition: None,
        },
        Provenance::source(DISK_MAIN, line),
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
            transition: None,
        },
        Provenance::runtime(key, RUNTIME_ENV),
    )
}
