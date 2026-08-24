use crate::capture_env::{
    RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR, RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_1_4_DEV,
};
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, BodyKind, DataAssumption, HardwareAssumption, HttpMethod, JourneyStep,
    Precondition, RouteRef, SoftwareAssumption, StepOutcome, UseCase, Visibility,
};
use crate::journey_presence::{
    PRESENCE_ADD_EXTERNAL_NMEA_GPS_SOCKET, PRESENCE_REMOVE_CONFIGURED_NMEA_SOCKET,
    PRESENCE_VIEW_CONFIGURED_NMEA_SOCKETS,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const NMEA_MAIN: &str = "core/services/nmea_injector/main.py";
const NMEA_MENUS: &str = "core/frontend/src/menus.ts";
const NMEA_INJECTOR: &str = "core/frontend/src/components/nmea-injector/NMEAInjector.vue";
const NMEA_CREATE_DIALOG: &str =
    "core/frontend/src/components/nmea-injector/NMEASocketCreationDialog.vue";
const NMEA_SOCKET_CARD: &str = "core/frontend/src/components/nmea-injector/NMEASocketCard.vue";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
const RUNTIME_ENV_1_4_DEV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_1_4_DEV;
const NMEA_TRANSITIONS_1_4_DEV: &str =
    "runtime-captures/nmea_injector__pi4_navigator_1_4_dev.json#transitions";

pub const JOURNEYS: &[UseCase] = &[
    VIEW_CONFIGURED_NMEA_SOCKETS,
    ADD_EXTERNAL_NMEA_GPS_SOCKET,
    REMOVE_CONFIGURED_NMEA_SOCKET,
];

const VIEW_CONFIGURED_NMEA_SOCKETS: UseCase = UseCase {
    id: JourneyId::ViewConfiguredNmeaSockets,
    summary: Grounded::known(
        "View configured NMEA input sockets and their MAVLink component mappings",
        Provenance::doc(
            ADV,
            543,
            "- Setup requires a UDP or TCP socket for the NMEA device to ",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 533, "{% pirate() %}"),
    ),
    services: NMEA_INJECTOR_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ListNmeaSockets,
        "NMEA Injector page lists sockets with kind, port, and MAVLink component ID",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareAssumption::AdvancedMode),
        Provenance::source(NMEA_MENUS, 80, "advanced: true,"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the NMEA Injector page from the sidebar",
            None,
            Provenance::source(NMEA_MENUS, 77, "title: 'MAVLink Inspector',"),
            None,
        ),
        operator_step(
            "View configured NMEA sockets with transport kind, port, and MAVLink component ID",
            Some(sourced_route(
                HttpMethod::Get,
                "/socks",
                Some("v1.0"),
                40,
                "@app.get(\"/socks\", response_model=List[NMEASocket])",
            )),
            Provenance::source(
                NMEA_INJECTOR,
                138,
                "async fetchAvailableNMEASockets(): Promise<void> {",
            ),
            Some(runtime_outcome(
                200,
                Some("[]"),
                BodyKind::Payload,
                "runtime-captures/nmea_injector__pi4_navigator_master.json#running_baseline",
                RUNTIME_ENV,
            )),
        ),
    ]),
    availability: PRESENCE_VIEW_CONFIGURED_NMEA_SOCKETS,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted("GET /socks only lists configured NMEA listeners without writes"),
    ),
    chains_from: None,
};

const ADD_EXTERNAL_NMEA_GPS_SOCKET: UseCase = UseCase {
    id: JourneyId::AddExternalNmeaGpsSocket,
    summary: Grounded::known(
        "Add a UDP or TCP socket so an external NMEA GPS device can inject positions as MAVLink",
        Provenance::doc(
            ADV,
            539,
            "- Conveys GPS positions (from an NMEA device) to the vehicle",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 533, "{% pirate() %}"),
    ),
    services: NMEA_INJECTOR_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::CreateNmeaSocket,
        "creation dialog submits socket kind, port, and MAVLink component ID to start listening",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Software(SoftwareAssumption::AdvancedMode),
            Provenance::source(NMEA_MENUS, 80, "advanced: true,"),
        ),
        GroundedItem::new(
            Precondition::Hardware(HardwareAssumption::ExternalNmeaGps),
            Provenance::doc(
                ADV,
                539,
                "- Conveys GPS positions (from an NMEA device) to the vehicle",
            ),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the NMEA Injector page from the sidebar",
            None,
            Provenance::source(NMEA_MENUS, 77, "title: 'MAVLink Inspector',"),
            None,
        ),
        operator_step(
            "Click the + button to open the new NMEA socket dialog",
            None,
            Provenance::source(NMEA_INJECTOR, 86, "@click=\"openCreationDialog\""),
            None,
        ),
        operator_step(
            "Choose socket kind (UDP or TCP), port, and MAVLink component ID",
            None,
            Provenance::doc(
                ADV,
                543,
                "- Setup requires a UDP or TCP socket for the NMEA device to ",
            ),
            None,
        ),
        operator_step(
            "Click Create to add the listening socket",
            Some(sourced_route(
                HttpMethod::Post,
                "/socks",
                Some("v1.0"),
                48,
                "@app.post(",
            )),
            Provenance::source(
                NMEA_CREATE_DIALOG,
                139,
                "nmea_injector.createNMEASocket(this.nmea_socket)",
            ),
            Some(runtime_outcome(
                201,
                Some("null"),
                BodyKind::Payload,
                NMEA_TRANSITIONS_1_4_DEV,
                RUNTIME_ENV_1_4_DEV,
            )),
        ),
    ]),
    availability: PRESENCE_ADD_EXTERNAL_NMEA_GPS_SOCKET,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "POST /socks opens a listening NMEA socket removable via DELETE /socks",
        ),
    ),
    chains_from: None,
};

const REMOVE_CONFIGURED_NMEA_SOCKET: UseCase = UseCase {
    id: JourneyId::RemoveConfiguredNmeaSocket,
    summary: Grounded::known(
        "Remove a configured NMEA input socket from the NMEA Injector",
        Provenance::source(
            NMEA_SOCKET_CARD,
            55,
            "nmea_injector.removeNMEASocket(this.nmeaSocket)",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 533, "{% pirate() %}"),
    ),
    services: NMEA_INJECTOR_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RemoveNmeaSocket,
        "socket card remove button deletes the matching kind, port, and component ID",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Software(SoftwareAssumption::AdvancedMode),
            Provenance::source(NMEA_MENUS, 80, "advanced: true,"),
        ),
        GroundedItem::new(
            Precondition::Data(DataAssumption::NmeaSocketConfigured),
            Provenance::source(
                NMEA_INJECTOR,
                71,
                "No NMEA sockets available. You can add a connection by click",
            ),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the NMEA Injector page from the sidebar",
            None,
            Provenance::source(NMEA_MENUS, 77, "title: 'MAVLink Inspector',"),
            None,
        ),
        operator_step(
            "View the configured NMEA socket to remove",
            Some(sourced_route(
                HttpMethod::Get,
                "/socks",
                Some("v1.0"),
                40,
                "@app.get(\"/socks\", response_model=List[NMEASocket])",
            )),
            Provenance::source(
                NMEA_INJECTOR,
                138,
                "async fetchAvailableNMEASockets(): Promise<void> {",
            ),
            Some(runtime_outcome(
                200,
                Some("[]"),
                BodyKind::Payload,
                "runtime-captures/nmea_injector__pi4_navigator_master.json#running_baseline",
                RUNTIME_ENV,
            )),
        ),
        operator_step(
            "Click the remove button on the socket card",
            Some(sourced_route(
                HttpMethod::Delete,
                "/socks",
                Some("v1.0"),
                59,
                "@app.delete(",
            )),
            Provenance::source(
                NMEA_SOCKET_CARD,
                55,
                "nmea_injector.removeNMEASocket(this.nmeaSocket)",
            ),
            Some(runtime_outcome(
                200,
                Some("null"),
                BodyKind::Payload,
                NMEA_TRANSITIONS_1_4_DEV,
                RUNTIME_ENV_1_4_DEV,
            )),
        ),
    ]),
    availability: PRESENCE_REMOVE_CONFIGURED_NMEA_SOCKET,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted("DELETE /socks removes one NMEA listener recreatable via POST /socks"),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const NMEA_INJECTOR_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::NmeaInjector,
    Provenance::doc(
        ADV,
        536,
        "{{ service(service=\"NMEA Injector\", port=2748, link=\"/servic",
    ),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::NmeaInjector,
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
        Provenance::source(NMEA_MAIN, line, anchor),
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

const fn runtime_outcome(
    status: u16,
    body: Option<&'static str>,
    body_kind: BodyKind,
    key: &'static str,
    env: &'static str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            body_kind,
            transition: None,
        },
        Provenance::runtime(key, env),
    )
}
