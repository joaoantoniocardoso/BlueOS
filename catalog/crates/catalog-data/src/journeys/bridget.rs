use crate::journey_presence::{
    PRESENCE_CREATE_SERIAL_TO_UDP_BRIDGE, PRESENCE_REMOVE_SERIAL_BRIDGE,
    PRESENCE_VIEW_CONFIGURED_SERIAL_BRIDGES,
};
use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, BodyKind, DataAssumption, HardwareAssumption, HttpMethod, JourneyStep,
    Precondition, RouteRef, SoftwareAssumption, StepOutcome, UseCase, Visibility,
};

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const BRIDGET_MAIN: &str = "core/services/bridget/main.py";
const BRIDGET_CORE: &str = "core/services/bridget/bridget.py";
const BRIDGET_MENUS: &str = "core/frontend/src/menus.ts";
const BRIDGET_STORE: &str = "core/frontend/src/store/bridget.ts";
const BRIDGET_VIEW: &str = "core/frontend/src/components/bridges/Bridget.vue";
const BRIDGET_CREATE_DIALOG: &str = "core/frontend/src/components/bridges/BridgeCreationDialog.vue";
const BRIDGET_CARD: &str = "core/frontend/src/components/bridges/BridgeCard.vue";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const JOURNEYS: &[UseCase] = &[
    VIEW_CONFIGURED_SERIAL_BRIDGES,
    CREATE_SERIAL_TO_UDP_BRIDGE,
    REMOVE_SERIAL_BRIDGE,
];

const VIEW_CONFIGURED_SERIAL_BRIDGES: UseCase =
    UseCase {
        id: JourneyId::ViewConfiguredSerialBridges,
        summary: Grounded::known(
            "View and manage configured bridges between serial and UDP/TCP endpoints",
            Provenance::doc(OVERVIEW, 132, "| [**Serial Bridges**](../advanced/#serial-bridges) | &rarr;"),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 568, "{% pirate() %}")),
        services: BRIDGET_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ListConfiguredSerialBridges,
            "Serial Bridges page lists configured bridges and available serial ports",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Software(SoftwareAssumption::AdvancedMode),
            Provenance::source(BRIDGET_MENUS, 102, "text: 'Manage detected Ping family sonar devices, connected "),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Serial Bridges page from the sidebar",
                None,
                Provenance::source(BRIDGET_MENUS, 99, "icon: 'mdi-radar',"),
                None,
            ),
            operator_step(
                "Load the list of configured serial bridges",
                Some(sourced_route(HttpMethod::Get, "/bridges", Some("v1.0"), 40, "@app.get(\"/bridges\", response_m")),
                Provenance::source(BRIDGET_STORE, 88, "url: `${this.API_URL}/bridges`,"),
                Some(runtime_outcome(
                    200,
                    Some("[] (empty; no bridges configured)"),
                    BodyKind::Payload,
                    "runtime-captures/bridget__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Load available serial ports for bridge creation",
                Some(sourced_route(HttpMethod::Get, "/serial_ports", Some("v1.0"), 32, "@app.get(\"/serial_ports\", ")),
                Provenance::source(BRIDGET_STORE, 112, "url: `${this.API_URL}/serial_ports`,"),
                Some(runtime_outcome(
                    200,
                    Some("[\"/dev/ttyAMA0\", \"/dev/ttyAMA1\", \"/dev/ttyAMA2\", \"/dev/ttyAMA3\", \"/dev/ttyS0\"] (proxied from linux2rest localhost:6030/serial)"),
                    BodyKind::Payload,
                    "runtime-captures/bridget__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "View the bridge card showing the serial device name and baud rate",
                None,
                Provenance::source(BRIDGET_CARD, 22, "{{ get_display_name(bridgeSerialInfo) }}:{{ bridgeSerialInfo"),
                None,
            ),
        ]),
        availability: PRESENCE_VIEW_CONFIGURED_SERIAL_BRIDGES,
        blast_radius: Grounded::known(
            BlastRadius::Safe,
            Provenance::asserted("Lists bridges and serial ports via read-only GET routes"),
        ),
        chains_from: None,
    };

const CREATE_SERIAL_TO_UDP_BRIDGE: UseCase =
    UseCase {
        id: JourneyId::CreateSerialToUdpBridge,
        summary: Grounded::known(
            "Create a high-performance link between a serial device connected to the onboard computer and a UDP port"
                ,
            Provenance::doc(ADV, 574, "The Serial Bridges page allows creating high performance lin"),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 572, "{% pirate() %}")),
        services: BRIDGET_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::CreateSerialToUdpBridge,
            "creation dialog submits serial path, baud, IP, and UDP ports to start a bridge",
        )]),
        preconditions: GroundedSet::known(&[
            GroundedItem::new(
                Precondition::Software(SoftwareAssumption::AdvancedMode),
                Provenance::source(BRIDGET_MENUS, 102, "text: 'Manage detected Ping family sonar devices, connected "),
            ),
            GroundedItem::new(
                Precondition::Hardware(HardwareAssumption::UsbSerialDevice),
                Provenance::doc(ADV, 574, "The Serial Bridges page allows creating high performance lin"),
            ),
        ]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Serial Bridges page from the sidebar",
                None,
                Provenance::source(BRIDGET_MENUS, 99, "icon: 'mdi-radar',"),
                None,
            ),
            operator_step(
                "Click the + button to open the new bridge dialog",
                None,
                Provenance::source(BRIDGET_VIEW, 52, "@click=\"openCreationDialog\""),
                None,
            ),
            operator_step(
                "Select the serial port for the bridge",
                None,
                Provenance::source(BRIDGET_CREATE_DIALOG, 17, "<v-select"),
                None,
            ),
            operator_step(
                "Select the serial baudrate",
                None,
                Provenance::source(BRIDGET_CREATE_DIALOG, 81, "<v-select"),
                None,
            ),
            operator_step(
                "Choose the UDP endpoint mode (server or client)",
                None,
                Provenance::source(BRIDGET_CREATE_DIALOG, 87, "<v-tabs"),
                None,
            ),
            operator_step(
                "Enter the UDP endpoint IP address and port",
                None,
                Provenance::source(BRIDGET_CREATE_DIALOG, 111, "<v-text-field"),
                None,
            ),
            operator_step(
                "Click Create to add the serial-to-UDP bridge",
                Some(sourced_route(HttpMethod::Post, "/bridges", Some("v1.0"), 48, "@app.post(\"/bridges\", status_c")),
                Provenance::source(BRIDGET_CREATE_DIALOG, 149, "@click=\"createBridge\""),
                Some(pending_outcome(
                    "POST /bridges bridge creation requires runtime capture with serial hardware attached (mutating; not exercised)",
                )),
            ),
            service_step(
                "Spawn a bridges process for the serial-to-UDP link",
                None,
                Provenance::source(BRIDGET_CORE, 75, "new_bridge = Bridge("),
                Some(pending_outcome(
                    "bridges subprocess startup and UDP endpoint assignment require runtime capture with serial hardware attached",
                )),
            ),
            service_step(
                "Persist bridge configuration to userdata settings",
                None,
                Provenance::source(BRIDGET_CORE, 85, "if settings_spec not in self._settings_manager.settings.spec"),
                None,
            ),
        ]),
        availability: PRESENCE_CREATE_SERIAL_TO_UDP_BRIDGE,
        blast_radius: Grounded::known(
            BlastRadius::Disruptive,
            Provenance::asserted(
                "Claims a serial device and spawns a bridges process that can contend with the autopilot link",
            ),
        ),
        chains_from: None,
    };

const REMOVE_SERIAL_BRIDGE: UseCase = UseCase {
    id: JourneyId::RemoveSerialBridge,
    summary: Grounded::known(
        "Remove a configured serial bridge from the Serial Bridges page",
        Provenance::source(BRIDGET_CARD, 60, "@click=\"removeBridge\""),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 568, "{% pirate() %}")),
    services: BRIDGET_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RemoveSerialBridge,
        "bridge card remove button deletes the matching serial path and UDP endpoint",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Software(SoftwareAssumption::AdvancedMode),
            Provenance::source(BRIDGET_MENUS, 102, "text: 'Manage detected Ping family sonar devices, connected "),
        ),
        GroundedItem::new(
            Precondition::Data(DataAssumption::SerialBridgeConfigured),
            Provenance::source(BRIDGET_VIEW, 8, "v-if=\"are_bridges_available && !updating_bridges\""),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Serial Bridges page from the sidebar",
            None,
            Provenance::source(BRIDGET_MENUS, 99, "icon: 'mdi-radar',"),
            None,
        ),
        operator_step(
            "View the configured bridge to remove",
            Some(sourced_route(HttpMethod::Get, "/bridges", Some("v1.0"), 40, "@app.get(\"/bridges\", response_model")),
            Provenance::source(BRIDGET_VIEW, 21, "<bridge-card :bridge-serial-info=\"info\" />"),
            Some(runtime_outcome(
                200,
                Some("[] (empty; no bridges configured)"),
                BodyKind::Payload,
                "runtime-captures/bridget__pi4_navigator_master.json#running_baseline",
            )),
        ),
        operator_step(
            "Click the remove button on the bridge card",
            Some(sourced_route(HttpMethod::Delete, "/bridges", Some("v1.0"), 56, "@app.delete(\"/bridges\", status_c")),
            Provenance::source(BRIDGET_CARD, 60, "@click=\"removeBridge\""),
            Some(source_outcome(200, 56, "@app.delete(\"/bridges\", status_code=status.HTTP_200_OK)")),
        ),
    ]),
    availability: PRESENCE_REMOVE_SERIAL_BRIDGE,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted("Deleting a configured bridge stops the subprocess and can be recreated from the dialog"),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const BRIDGET_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Bridget,
    Provenance::doc(
        ADV,
        571,
        "{{ service(service=\"Bridget\", port=27353, link=\"/services/br",
    ),
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
    anchor: &'static str,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(BRIDGET_MAIN, line, anchor),
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

const fn source_outcome(status: u16, line: u32, anchor: &'static str) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind: BodyKind::Unknown,
            transition: None,
        },
        Provenance::source(BRIDGET_MAIN, line, anchor),
    )
}

const fn runtime_outcome(
    status: u16,
    body: Option<&'static str>,
    body_kind: BodyKind,
    key: &'static str,
) -> Grounded<StepOutcome> {
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
