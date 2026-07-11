use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const SYSTEM_INFO_MENUS: &str = "core/frontend/src/menus.ts";
const SYSTEM_INFO_STORE: &str = "core/frontend/src/store/system-information.ts";
const SYSTEM_INFO_VIEW: &str = "core/frontend/src/views/SystemInformationView.vue";
const SYSTEM_CONDITION: &str =
    "core/frontend/src/components/system-information/SystemCondition.vue";
const RUNTIME_CAPTURE: &str = "runtime-captures/linux2rest__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub fn journeys() -> Vec<UserJourney> {
    vec![view_system_information()]
}

fn view_system_information() -> UserJourney {
    UserJourney {
        id: JourneyId("view_system_information".into()),
        summary: Grounded::known(
            "View live hardware and system status (CPU, memory, disk, network, processes)".into(),
            Provenance::doc(OVERVIEW, 124),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(SYSTEM_INFO_MENUS, 110)),
        services: linux2rest_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "view_system_information",
            "System Information page and System Monitor tab show live CPU, memory, disk, and temperature backed by linux2rest",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the System Information page from the sidebar",
                None,
                Provenance::source(SYSTEM_INFO_MENUS, 107),
                None,
            ),
            operator_step(
                "Load the System Information interface with monitor, process, and network tabs",
                None,
                Provenance::source(SYSTEM_INFO_VIEW, 74),
                None,
            ),
            operator_step(
                "View live CPU, memory, disk, and temperature cards on the System Monitor tab",
                None,
                Provenance::source(SYSTEM_CONDITION, 4),
                None,
            ),
            operator_step(
                "Load the system snapshot used by the monitor widgets",
                Some(sourced_route(HttpMethod::Get, "/system", None, 31)),
                Provenance::source(SYSTEM_INFO_STORE, 241),
                Some(runtime_outcome(200, Some("\"cpu\"".into()), "#running_baseline")),
            ),
            operator_step(
                "Periodically refresh CPU, memory, and disk metrics",
                Some(sourced_route(HttpMethod::Get, "/system/cpu", None, 33)),
                Provenance::source(SYSTEM_CONDITION, 135),
                Some(runtime_outcome(
                    200,
                    Some("\"name\":\"cpu0\"".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "View processes, network configuration, and other system details on the remaining tabs",
                None,
                Provenance::doc(ADV, 602),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn linux2rest_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::Linux2rest,
        Provenance::doc(ADV, 600),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Linux2rest,
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
        Provenance::source(SYSTEM_INFO_STORE, line),
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
