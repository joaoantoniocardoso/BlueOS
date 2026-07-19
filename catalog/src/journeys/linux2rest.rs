use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use crate::version::FeatureAvailability;

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const SYSTEM_INFO_MENUS: &str = "core/frontend/src/menus.ts";
const SYSTEM_INFO_STORE: &str = "core/frontend/src/store/system-information.ts";
const SYSTEM_INFO_VIEW: &str = "core/frontend/src/views/SystemInformationView.vue";
const SYSTEM_CONDITION: &str =
    "core/frontend/src/components/system-information/SystemCondition.vue";
// capture used RepoDigest as primary
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub const JOURNEYS: &[UserJourney] = &[VIEW_SYSTEM_INFORMATION];

const VIEW_SYSTEM_INFORMATION: UserJourney =
    UserJourney {
        id: JourneyId::ViewSystemInformation,
        summary: Grounded::known(
            "View live hardware and system status (CPU, memory, disk, network, processes)",
            Provenance::doc(OVERVIEW, 124),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(SYSTEM_INFO_MENUS, 110)),
        services: LINUX2REST_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ViewSystemInformation,
            "System Information page and System Monitor tab show live CPU, memory, disk, and temperature backed by linux2rest",
        )]),
        preconditions: GroundedSet::known(&[]),
        steps: GroundedSet::known(&[
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
                Some(runtime_outcome(200, Some("\"cpu\""), "runtime-captures/linux2rest__pi4_navigator_master.json#running_baseline")),
            ),
            operator_step(
                "Periodically refresh CPU, memory, and disk metrics",
                Some(sourced_route(HttpMethod::Get, "/system/cpu", None, 33)),
                Provenance::source(SYSTEM_CONDITION, 135),
                Some(runtime_outcome(
                    200,
                    Some("\"name\":\"cpu0\""),
                    "runtime-captures/linux2rest__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "View processes, network configuration, and other system details on the remaining tabs",
                None,
                Provenance::doc(ADV, 602),
                None,
            ),
        ]),
        availability: FeatureAvailability::unknown(),
    chains_from: None,
    };

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const LINUX2REST_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Linux2rest,
    Provenance::doc(ADV, 600),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Linux2rest,
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
        Provenance::source(SYSTEM_INFO_STORE, line),
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
