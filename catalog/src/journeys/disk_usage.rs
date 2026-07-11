use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const GS: &str = "content/usage/getting-started/index.md";
const DISK_MAIN: &str = "core/services/disk_usage/main.py";
const DISK_MENUS: &str = "core/frontend/src/menus.ts";

#[allow(dead_code)]
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
        id: JourneyId("inspect_disk_usage".into()),
        summary: Grounded::known(
            "Visualize disk usage as a directory tree and navigate into folders to find what consumes storage"
                .into(),
            Provenance::source(DISK_MENUS, 51),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
        services: disk_usage_services(),
        capability_refs: GroundedSet::known(vec![
            cap(
                "inspect_disk_usage",
                "Disk Usage tab loads a du-backed usage tree for the selected path",
            ),
            cap(
                "navigate_disk_usage",
                "operator drills into subdirectories by re-fetching usage for a new path",
            ),
        ]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Disk page from the sidebar",
                None,
                Provenance::doc(ADV, 188),
            ),
            operator_step(
                "Load the disk usage tree for the current path",
                Some(sourced_route(HttpMethod::Get, "/disk/usage", Some("v1.0"), 248)),
                Provenance::source(DISK_MAIN, 251),
            ),
            operator_step(
                "Open a subdirectory to inspect how storage is distributed beneath it",
                Some(sourced_route(HttpMethod::Get, "/disk/usage", Some("v1.0"), 248)),
                Provenance::source(DISK_MAIN, 255),
            ),
        ]),
        chains_from: None,
    }
}

fn free_disk_space() -> UserJourney {
    UserJourney {
        id: JourneyId("free_disk_space".into()),
        summary: Grounded::known(
            "Delete files and folders to free up storage space on the onboard computer".into(),
            Provenance::doc(GS, 65),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
        services: disk_usage_services(),
        capability_refs: GroundedSet::known(vec![
            cap(
                "inspect_disk_usage",
                "operator browses the usage tree to choose deletion targets",
            ),
            cap(
                "delete_disk_paths",
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
                Provenance::doc(ADV, 188),
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
            ),
            operator_step(
                "Select one or more paths to delete",
                None,
                Provenance::source(DISK_MAIN, 270),
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
            ),
        ]),
        chains_from: Some(JourneyId("inspect_disk_usage".into())),
    }
}

fn run_single_disk_speed_test() -> UserJourney {
    UserJourney {
        id: JourneyId("run_single_disk_speed_test".into()),
        summary: Grounded::known(
            "Run a single-size disk read/write speed benchmark using the disktest binary".into(),
            Provenance::source(DISK_MAIN, 413),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
        services: disk_usage_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "run_disk_speed_test",
            "GET /disk/speed runs one disktest write-and-verify pass at the requested size",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("disktest binary is available on PATH".into()),
            Provenance::source(DISK_MAIN, 323),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Disk page and switch to the Speed Test tab",
                None,
                Provenance::source(DISK_MENUS, 51),
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
            ),
        ]),
        chains_from: None,
    }
}

fn run_multi_size_disk_speed_test() -> UserJourney {
    UserJourney {
        id: JourneyId("run_multi_size_disk_speed_test".into()),
        summary: Grounded::known(
            "Run a progressive multi-size disk speed benchmark with streaming results".into(),
            Provenance::source(DISK_MAIN, 446),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(DISK_MENUS, 50)),
        services: disk_usage_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "run_multi_size_disk_speed_test",
            "GET /disk/speed/stream yields NDJSON points for each benchmark size",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("disktest binary is available on PATH".into()),
            Provenance::source(DISK_MAIN, 323),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Disk page and switch to the Speed Test tab",
                None,
                Provenance::source(DISK_MENUS, 51),
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
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn disk_usage_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId("disk_usage".into()),
        Provenance::source(DISK_MAIN, 26),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId("disk_usage".into()),
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
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description: description.into(),
            route,
            outcome: None,
        },
        provenance,
    )
}
