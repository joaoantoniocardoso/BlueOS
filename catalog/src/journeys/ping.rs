use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
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
const RUNTIME_CAPTURE: &str = "runtime-captures/ping__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn journeys() -> Vec<UserJourney> {
    vec![
        view_detected_sonar_devices(),
        connect_ping_viewer_to_sonar(),
        enable_ping1d_rangefinder_mavlink(),
    ]
}

fn view_detected_sonar_devices() -> UserJourney {
    UserJourney {
        id: JourneyId::ViewDetectedSonarDevices,
        summary: Grounded::known(
            "View Ping family sonar devices auto-detected on serial/USB and the local network".into(),
            Provenance::doc(ADV, 553),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 548)),
        services: ping_services(),
        capability_refs: GroundedSet::known(vec![cap(CapabilityId::ListDetectedPingSensors,
            "Ping Sonar Devices page lists auto-detected Ping1D and Ping360 sensors via GET /sensors",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            service_step(
                "Probe serial ports for Ping-protocol devices",
                None,
                Provenance::source(PING_PROBER, 13),
                None,
            ),
            service_step(
                "Discover ethernet-configured Ping360 devices on the local network",
                None,
                Provenance::source(PING360_ETH_PROBER, 43),
                None,
            ),
            service_step(
                "Spawn a UDP bridge for each detected sonar device",
                None,
                Provenance::source(PING_MANAGER, 43),
                Some(pending_outcome(
                    "UDP bridge port assignment requires runtime capture with Ping sonar hardware attached",
                )),
            ),
            operator_step(
                "Open the Ping Sonar Devices page from the sidebar",
                None,
                Provenance::source(PING_MENUS, 91),
                None,
            ),
            operator_step(
                "Load the list of detected Ping sensors",
                Some(sourced_route(HttpMethod::Get, "/sensors", Some("v1.0"), 38)),
                Provenance::source(PING_STORE, 63),
                Some(runtime_outcome(
                    200,
                    Some("[] (empty; no Ping sonar hardware attached)".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "View the device card showing the sonar type and its UDP bridge endpoint",
                None,
                Provenance::source(PING1D_CARD, 15),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn connect_ping_viewer_to_sonar() -> UserJourney {
    UserJourney {
        id: JourneyId::ConnectPingViewerToSonar,
        summary: Grounded::known(
            "Connect Ping Viewer on the surface computer to a vehicle-exposed Ping sonar".into(),
            Provenance::doc(OVERVIEW, 131),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(PING_MENUS, 94)),
        services: ping_services(),
        capability_refs: GroundedSet::known(vec![cap(CapabilityId::ConnectPingViewerToSonar,
            "operator uses the UDP bridge port shown on the device card to reach the sonar from Ping Viewer",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::HardwarePresent("Ping family sonar device".into()),
            Provenance::doc(ADV, 553),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Ping Sonar Devices page from the sidebar",
                None,
                Provenance::source(PING_MENUS, 91),
                None,
            ),
            operator_step(
                "Note the connection endpoint shown on the device card (UDP bridge port or ethernet IP)",
                None,
                Provenance::source(PING360_CARD, 15),
                None,
            ),
            operator_step(
                "Connect Ping Viewer on the surface computer to the sonar",
                None,
                Provenance::doc(OVERVIEW, 131),
                Some(pending_outcome(
                    "Ping Viewer sonar connection requires runtime capture with Ping sonar hardware attached",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn enable_ping1d_rangefinder_mavlink() -> UserJourney {
    UserJourney {
        id: JourneyId::EnablePing1dRangefinderMavlink,
        summary: Grounded::known(
            "Enable Ping1D distance estimates as MAVLink DISTANCE_SENSOR messages to the autopilot"
                .into(),
            Provenance::doc(ADV, 559),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 548)),
        services: ping_services(),
        capability_refs: GroundedSet::known(vec![cap(CapabilityId::EnablePing1dMavlinkDistance,
            "Ping1D card MAVLink Distances switch posts sensor settings to toggle mavlink_driver",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::HardwarePresent("Ping sonar (Ping1D)".into()),
            Provenance::doc(ADV, 559),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Ping Sonar Devices page from the sidebar",
                None,
                Provenance::source(PING_MENUS, 91),
                None,
            ),
            operator_step(
                "Toggle the MAVLink Distances switch on the Ping1D device card",
                None,
                Provenance::source(PING1D_CARD, 21),
                None,
            ),
            operator_step(
                "Submit updated sensor settings to enable MAVLink distance forwarding",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/sensors",
                    Some("v1.0"),
                    46,
                )),
                Provenance::source(PING1D_CARD, 97),
                Some(pending_outcome(
                    "MAVLink DISTANCE_SENSOR forwarding requires runtime capture with Ping1D hardware and POST /sensors (mutating; not exercised)",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: CapabilityId, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

fn ping_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::Ping,
        Provenance::doc(ADV, 549),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Ping,
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
        Provenance::source(PING_MAIN, line),
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
            actor: Actor::Service(ServiceId::Ping),
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
