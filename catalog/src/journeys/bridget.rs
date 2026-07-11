use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const BRIDGET_MAIN: &str = "core/services/bridget/main.py";
const BRIDGET_CORE: &str = "core/services/bridget/bridget.py";
const BRIDGET_MENUS: &str = "core/frontend/src/menus.ts";
const BRIDGET_STORE: &str = "core/frontend/src/store/bridget.ts";
const BRIDGET_VIEW: &str = "core/frontend/src/components/bridges/Bridget.vue";
const BRIDGET_CREATE_DIALOG: &str = "core/frontend/src/components/bridges/BridgeCreationDialog.vue";
const BRIDGET_CARD: &str = "core/frontend/src/components/bridges/BridgeCard.vue";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub const JOURNEYS: &[UserJourney] = &[
    VIEW_CONFIGURED_SERIAL_BRIDGES,
    CREATE_SERIAL_TO_UDP_BRIDGE,
    REMOVE_SERIAL_BRIDGE,
];

const VIEW_CONFIGURED_SERIAL_BRIDGES: UserJourney =
    UserJourney {
        id: JourneyId::ViewConfiguredSerialBridges,
        summary: Grounded::known(
            "View and manage configured bridges between serial and UDP/TCP endpoints",
            Provenance::doc(OVERVIEW, 132),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 568)),
        services: BRIDGET_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ListConfiguredSerialBridges,
            "Serial Bridges page lists configured bridges and available serial ports",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Other("Advanced mode enabled to access the Serial Bridges page"),
            Provenance::source(BRIDGET_MENUS, 102),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Serial Bridges page from the sidebar",
                None,
                Provenance::source(BRIDGET_MENUS, 99),
                None,
            ),
            operator_step(
                "Load the list of configured serial bridges",
                Some(sourced_route(HttpMethod::Get, "/bridges", Some("v1.0"), 40)),
                Provenance::source(BRIDGET_STORE, 88),
                Some(runtime_outcome(
                    200,
                    Some("[] (empty; no bridges configured)"),
                    "runtime-captures/bridget__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Load available serial ports for bridge creation",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/serial_ports",
                    Some("v1.0"),
                    32,
                )),
                Provenance::source(BRIDGET_STORE, 112),
                Some(runtime_outcome(
                    200,
                    Some("[\"/dev/ttyAMA0\", \"/dev/ttyAMA1\", \"/dev/ttyAMA2\", \"/dev/ttyAMA3\", \"/dev/ttyS0\"] (proxied from linux2rest localhost:6030/serial)"),
                    "runtime-captures/bridget__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "View the bridge card showing the serial device name and baud rate",
                None,
                Provenance::source(BRIDGET_CARD, 22),
                None,
            ),
        ]),
        chains_from: None,
    };

const CREATE_SERIAL_TO_UDP_BRIDGE: UserJourney =
    UserJourney {
        id: JourneyId::CreateSerialToUdpBridge,
        summary: Grounded::known(
            "Create a high-performance link between a serial device connected to the onboard computer and a UDP port"
                ,
            Provenance::doc(ADV, 574),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 572)),
        services: BRIDGET_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::CreateSerialToUdpBridge,
            "creation dialog submits serial path, baud, IP, and UDP ports to start a bridge",
        )]),
        preconditions: GroundedSet::known(&[
            GroundedItem::new(
                Precondition::Other("Advanced mode enabled to access the Serial Bridges page"),
                Provenance::source(BRIDGET_MENUS, 102),
            ),
            GroundedItem::new(
                Precondition::HardwarePresent(
                    "Serial device connected to the onboard computer",
                ),
                Provenance::doc(ADV, 574),
            ),
        ]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Serial Bridges page from the sidebar",
                None,
                Provenance::source(BRIDGET_MENUS, 99),
                None,
            ),
            operator_step(
                "Click the + button to open the new bridge dialog",
                None,
                Provenance::source(BRIDGET_VIEW, 52),
                None,
            ),
            operator_step(
                "Select the serial port for the bridge",
                None,
                Provenance::source(BRIDGET_CREATE_DIALOG, 17),
                None,
            ),
            operator_step(
                "Select the serial baudrate",
                None,
                Provenance::source(BRIDGET_CREATE_DIALOG, 81),
                None,
            ),
            operator_step(
                "Choose the UDP endpoint mode (server or client)",
                None,
                Provenance::source(BRIDGET_CREATE_DIALOG, 87),
                None,
            ),
            operator_step(
                "Enter the UDP endpoint IP address and port",
                None,
                Provenance::source(BRIDGET_CREATE_DIALOG, 111),
                None,
            ),
            operator_step(
                "Click Create to add the serial-to-UDP bridge",
                Some(sourced_route(HttpMethod::Post, "/bridges", Some("v1.0"), 48)),
                Provenance::source(BRIDGET_CREATE_DIALOG, 149),
                Some(pending_outcome(
                    "POST /bridges bridge creation requires runtime capture with serial hardware attached (mutating; not exercised)",
                )),
            ),
            service_step(
                "Spawn a bridges process for the serial-to-UDP link",
                None,
                Provenance::source(BRIDGET_CORE, 75),
                Some(pending_outcome(
                    "bridges subprocess startup and UDP endpoint assignment require runtime capture with serial hardware attached",
                )),
            ),
            service_step(
                "Persist bridge configuration to userdata settings",
                None,
                Provenance::source(BRIDGET_CORE, 85),
                None,
            ),
        ]),
        chains_from: None,
    };

const REMOVE_SERIAL_BRIDGE: UserJourney =
    UserJourney {
        id: JourneyId::RemoveSerialBridge,
        summary: Grounded::known(
            "Remove a configured serial bridge from the Serial Bridges page",
            Provenance::source(BRIDGET_CARD, 60),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 568)),
        services: BRIDGET_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::RemoveSerialBridge,
            "bridge card remove button deletes the matching serial path and UDP endpoint",
        )]),
        preconditions: GroundedSet::known(&[
            GroundedItem::new(
                Precondition::Other("Advanced mode enabled to access the Serial Bridges page"),
                Provenance::source(BRIDGET_MENUS, 102),
            ),
            GroundedItem::new(
                Precondition::Other("At least one serial bridge is already configured"),
                Provenance::source(BRIDGET_VIEW, 8),
            ),
        ]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Serial Bridges page from the sidebar",
                None,
                Provenance::source(BRIDGET_MENUS, 99),
                None,
            ),
            operator_step(
                "View the configured bridge to remove",
                Some(sourced_route(HttpMethod::Get, "/bridges", Some("v1.0"), 40)),
                Provenance::source(BRIDGET_VIEW, 21),
                Some(runtime_outcome(
                    200,
                    Some("[] (empty; no bridges configured)"),
                    "runtime-captures/bridget__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Click the remove button on the bridge card",
                Some(sourced_route(HttpMethod::Delete, "/bridges", Some("v1.0"), 56)),
                Provenance::source(BRIDGET_CARD, 60),
                Some(pending_outcome(
                    "DELETE /bridges bridge removal requires runtime capture with a configured bridge (mutating; not exercised)",
                )),
            ),
        ]),
        chains_from: None,
    };

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const BRIDGET_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Bridget,
    Provenance::doc(ADV, 571),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Bridget,
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
        Provenance::source(BRIDGET_MAIN, line),
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
            actor: Actor::Service(ServiceId::Bridget),
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
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            transition: None,
        },
        Provenance::runtime(key, RUNTIME_ENV),
    )
}
