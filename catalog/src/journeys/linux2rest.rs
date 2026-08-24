use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, BodyKind, HttpMethod, JourneyStep, RouteRef, StepOutcome, UseCase,
    Visibility,
};
use crate::journey_presence::PRESENCE_VIEW_SYSTEM_INFORMATION;
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const SYSTEM_INFO_MENUS: &str = "core/frontend/src/menus.ts";
const SYSTEM_INFO_STORE: &str = "core/frontend/src/store/system-information.ts";
const SYSTEM_INFO_VIEW: &str = "core/frontend/src/views/SystemInformationView.vue";
const SYSTEM_CONDITION: &str =
    "core/frontend/src/components/system-information/SystemCondition.vue";
// capture used RepoDigest as primary
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub const JOURNEYS: &[UseCase] = &[VIEW_SYSTEM_INFORMATION];

const VIEW_SYSTEM_INFORMATION: UseCase =
    UseCase {
        id: JourneyId::ViewSystemInformation,
        summary: Grounded::known(
            "View live hardware and system status (CPU, memory, disk, network, processes)",
            Provenance::doc(OVERVIEW, 124, "| [**System information**](../advanced/#system-information) "),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(SYSTEM_INFO_MENUS, 110, "text: 'Allows")),
        services: LINUX2REST_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ViewSystemInformation,
            "System Information page and System Monitor tab show live CPU, memory, disk, and temperature backed by linux2rest",
        )]),
        preconditions: GroundedSet::known(&[]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the System Information page from the sidebar",
                None,
                Provenance::source(SYSTEM_INFO_MENUS, 107, "icon: 'mdi-bridge',"),
                None,
            ),
            operator_step(
                "Load the System Information interface with monitor, process, and network tabs",
                None,
                Provenance::source(SYSTEM_INFO_VIEW, 74, "{ title: 'System Monitor', icon: 'mdi-speedometer', value"),
                None,
            ),
            operator_step(
                "View live CPU, memory, disk, and temperature cards on the System Monitor tab",
                None,
                Provenance::source(SYSTEM_CONDITION, 4, "v-for=\"(item, i) in [cpu, memory, disk, temperature]\""),
                None,
            ),
            operator_step(
                "Load the system snapshot used by the monitor widgets",
                Some(sourced_route(HttpMethod::Get, "/system", None, 31, "SystemType = 'system',")),
                Provenance::source(SYSTEM_INFO_STORE, 241, "await this.fetchSystemInformation(FetchType.SystemType)"),
                Some(runtime_outcome(200, Some("\"cpu\""), "runtime-captures/linux2rest__pi4_navigator_master.json#running_baseline")),
            ),
            operator_step(
                "Periodically refresh CPU, memory, and disk metrics",
                Some(sourced_route(HttpMethod::Get, "/system/cpu", None, 33, "SystemDiskType = 'system/disk',")),
                Provenance::source(SYSTEM_CONDITION, 135, "system_information.fetchSystemInformation(FetchType.Syst"),
                Some(runtime_outcome(
                    200,
                    Some("\"name\":\"cpu0\""),
                    "runtime-captures/linux2rest__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "View processes, network configuration, and other system details on the remaining tabs",
                None,
                Provenance::doc(ADV, 602, "The System Information page provides useful information abou"),
                None,
            ),
        ]),
        availability: PRESENCE_VIEW_SYSTEM_INFORMATION,
        blast_radius: Grounded::known(
            BlastRadius::Safe,
            Provenance::asserted(
                "System Information page only reads linux2rest /system and /system/cpu metrics",
            ),
        ),
        chains_from: None,
    };

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const LINUX2REST_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Linux2rest,
    Provenance::doc(
        ADV,
        600,
        "{{ service(service=\"System Information\", port=6030) }}",
    ),
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
    anchor: &'static str,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(SYSTEM_INFO_STORE, line, anchor),
    )
}

const fn runtime_body_kind(status: u16, body: Option<&'static str>) -> BodyKind {
    match body {
        Some(_) => BodyKind::Payload,
        None if status == 204 => BodyKind::Empty,
        None => BodyKind::Unknown,
    }
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
