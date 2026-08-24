use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, BodyKind, HardwareAssumption, HttpMethod, JourneyStep, Precondition,
    RouteRef, StepOutcome, UseCase, Visibility,
};
use crate::journey_presence::{
    PRESENCE_CONNECT_PING_VIEWER_TO_SONAR, PRESENCE_ENABLE_PING1D_RANGEFINDER_MAVLINK,
    PRESENCE_VIEW_DETECTED_SONAR_DEVICES,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const PING_MAIN: &str = "core/services/ping/main.py";
const PING_MANAGER: &str = "core/services/ping/pingmanager.py";
const PING_PROBER: &str = "core/services/ping/pingprober.py";
const PING360_ETH_PROBER: &str = "core/services/ping/ping360_ethernet_prober.py";
const PING_MENUS: &str = "core/frontend/src/menus.ts";
const PING_STORE: &str = "core/frontend/src/store/ping.ts";
const PING1D_CARD: &str = "core/frontend/src/components/ping/ping1d.vue";
const PING360_CARD: &str = "core/frontend/src/components/ping/ping360.vue";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const JOURNEYS: &[UseCase] = &[
    VIEW_DETECTED_SONAR_DEVICES,
    CONNECT_PING_VIEWER_TO_SONAR,
    ENABLE_PING1D_RANGEFINDER_MAVLINK,
];

const VIEW_DETECTED_SONAR_DEVICES: UseCase =
    UseCase {
        id: JourneyId::ViewDetectedSonarDevices,
        summary: Grounded::known(
            "View Ping family sonar devices auto-detected on serial/USB and the local network",
            Provenance::doc(ADV, 553, "The Ping Sonar Devices page shows any detected"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 548, "### Ping Sonar Devices")),
        services: PING_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ListDetectedPingSensors,
            "Ping Sonar Devices page lists auto-detected Ping1D and Ping360 sensors via GET /sensors",
        )]),
        preconditions: GroundedSet::known(&[]),
        steps: GroundedSet::known(&[
            service_step(
                "Probe serial ports for Ping-protocol devices",
                None,
                Provenance::source(PING_PROBER, 13, "async def probe(self, port: SysFS) -> Optional[PingDeviceDes"),
                None,
            ),
            service_step(
                "Discover ethernet-configured Ping360 devices on the local network",
                None,
                Provenance::source(PING360_ETH_PROBER, 43, "async def find_ping360_ethernet() -> List[PingDeviceDes"),
                None,
            ),
            service_step(
                "Spawn a UDP bridge for each detected sonar device",
                None,
                Provenance::source(PING_MANAGER, 43, "async def launch_driver_instance(self, ping: PingDeviceDescr"),
                Some(pending_outcome(
                    "UDP bridge port assignment requires runtime capture with Ping sonar hardware attached",
                )),
            ),
            operator_step(
                "Open the Ping Sonar Devices page from the sidebar",
                None,
                Provenance::source(PING_MENUS, 91, "title: 'Network Test',"),
                None,
            ),
            operator_step(
                "Load the list of detected Ping sensors",
                Some(sourced_route(HttpMethod::Get, "/sensors", Some("v1.0"), 38, "@app.get(\"/sensors\", response_m")),
                Provenance::source(PING_STORE, 63, "url: `${this.API_URL}/sensors`,"),
                Some(runtime_outcome(
                    200,
                    Some("[] (empty; no Ping sonar hardware attached)"),
                    "runtime-captures/ping__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "View the device card showing the sonar type and its UDP bridge endpoint",
                None,
                Provenance::source(PING1D_CARD, 15, "<tr><td>Bridge</td><td> UDP {{ device.driver_status.udp_port"),
                None,
            ),
        ]),
        availability: PRESENCE_VIEW_DETECTED_SONAR_DEVICES,
        blast_radius: Grounded::known(
            BlastRadius::Safe,
            Provenance::asserted("GET /sensors lists auto-detected Ping devices without operator writes"),
        ),
        chains_from: None,
    };

const CONNECT_PING_VIEWER_TO_SONAR: UseCase =
    UseCase {
        id: JourneyId::ConnectPingViewerToSonar,
        summary: Grounded::known(
            "Connect Ping Viewer on the surface computer to a vehicle-exposed Ping sonar",
            Provenance::doc(OVERVIEW, 131, "| [**Ping Sonar Devices**](../advanced/#ping-sonar-devices) "),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(PING_MENUS, 94, "show: true,")),
        services: PING_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ConnectPingViewerToSonar,
            "operator uses the UDP bridge port shown on the device card to reach the sonar from Ping Viewer",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::HardwarePresent("Ping family sonar device"),
            Provenance::doc(ADV, 553, "The Ping Sonar Devices page shows any detected"),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Ping Sonar Devices page from the sidebar",
                None,
                Provenance::source(PING_MENUS, 91, "title: 'Network Test',"),
                None,
            ),
            operator_step(
                "Note the connection endpoint shown on the device card (UDP bridge port or ethernet IP)",
                None,
                Provenance::source(PING360_CARD, 15, "<tr><td>{{ is_ethernet() ? \"IP\" : \"Bridge\" }}</td><td> {{ i"),
                None,
            ),
            operator_step(
                "Connect Ping Viewer on the surface computer to the sonar",
                None,
                Provenance::doc(OVERVIEW, 131, "| [**Ping Sonar Devices**](../advanced/#ping-sonar-devices) "),
                Some(pending_outcome(
                    "Ping Viewer sonar connection requires runtime capture with Ping sonar hardware attached",
                )),
            ),
        ]),
        availability: PRESENCE_CONNECT_PING_VIEWER_TO_SONAR,
        blast_radius: Grounded::known(
            BlastRadius::Safe,
            Provenance::asserted(
                "Operator reads the UDP bridge endpoint; Ping Viewer connects externally without vehicle API writes",
            ),
        ),
        chains_from: None,
    };

const ENABLE_PING1D_RANGEFINDER_MAVLINK: UseCase =
    UseCase {
        id: JourneyId::EnablePing1dRangefinderMavlink,
        summary: Grounded::known(
            "Enable Ping1D distance estimates as MAVLink DISTANCE_SENSOR messages to the autopilot"
                ,
            Provenance::doc(ADV, 559, "- Allows configuring Ping Sonar distance estimates to send a"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 548, "### Ping Sonar Devices")),
        services: PING_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::EnablePing1dMavlinkDistance,
            "Ping1D card MAVLink Distances switch posts sensor settings to toggle mavlink_driver",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Hardware(HardwareAssumption::Ping1d),
            Provenance::doc(ADV, 559, "- Allows configuring Ping Sonar distance estimates to send a"),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Ping Sonar Devices page from the sidebar",
                None,
                Provenance::source(PING_MENUS, 91, "title: 'Network Test',"),
                None,
            ),
            operator_step(
                "Toggle the MAVLink Distances switch on the Ping1D device card",
                None,
                Provenance::source(PING1D_CARD, 21, "<v-switch"),
                None,
            ),
            operator_step(
                "Submit updated sensor settings to enable MAVLink distance forwarding",
                Some(sourced_route(HttpMethod::Post, "/sensors", Some("v1.0"), 46, "@app.post(\"/sensors\", status_c")),
                Provenance::source(PING1D_CARD, 97, "async update_mavlink_driver() {"),
                Some(pending_outcome(
                    "MAVLink DISTANCE_SENSOR forwarding requires runtime capture with Ping1D hardware and POST /sensors (mutating; not exercised)",
                )),
            ),
        ]),
        availability: PRESENCE_ENABLE_PING1D_RANGEFINDER_MAVLINK,
        blast_radius: Grounded::known(
            BlastRadius::Reversible,
            Provenance::asserted(
                "POST /sensors toggles Ping1D MAVLink distance forwarding restorable by another POST",
            ),
        ),
        chains_from: None,
    };

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const PING_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Ping,
    Provenance::doc(
        ADV,
        549,
        "{{ service(service=\"Ping Service\", port=9110, link=\"/service",
    ),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Ping,
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
        Provenance::source(PING_MAIN, line, anchor),
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
            actor: Actor::Service(ServiceId::Ping),
            description,
            route,
            outcome,
        },
        provenance,
    )
}

const fn pending_outcome(reason: &'static str) -> Grounded<StepOutcome> {
    Grounded::unknown(reason)
}

const fn runtime_outcome(
    status: u16,
    body: Option<&'static str>,
    key: &'static str,
) -> Grounded<StepOutcome> {
    let body_kind = if body.is_some() {
        BodyKind::Payload
    } else {
        BodyKind::Unknown
    };
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            body_kind,
            transition: None,
        },
        Provenance::runtime(key, RUNTIME_ENV),
    )
}
