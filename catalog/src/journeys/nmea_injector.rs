use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, DataRequirement, HardwareRequirement, HttpMethod, JourneyStep, Precondition, RouteRef,
    SoftwareRequirement, StepOutcome, UserJourney, Visibility,
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

pub const JOURNEYS: &[UserJourney] = &[
    VIEW_CONFIGURED_NMEA_SOCKETS,
    ADD_EXTERNAL_NMEA_GPS_SOCKET,
    REMOVE_CONFIGURED_NMEA_SOCKET,
];

const VIEW_CONFIGURED_NMEA_SOCKETS: UserJourney = UserJourney {
    id: JourneyId::ViewConfiguredNmeaSockets,
    summary: Grounded::known(
        "View configured NMEA input sockets and their MAVLink component mappings",
        Provenance::doc(ADV, 543),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 533)),
    services: NMEA_INJECTOR_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ListNmeaSockets,
        "NMEA Injector page lists sockets with kind, port, and MAVLink component ID",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareRequirement::AdvancedMode),
        Provenance::source(NMEA_MENUS, 80),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the NMEA Injector page from the sidebar",
            None,
            Provenance::source(NMEA_MENUS, 77),
            None,
        ),
        operator_step(
            "View configured NMEA sockets with transport kind, port, and MAVLink component ID",
            Some(sourced_route(HttpMethod::Get, "/socks", Some("v1.0"), 40)),
            Provenance::source(NMEA_INJECTOR, 138),
            Some(runtime_outcome(
                200,
                Some("[]"),
                "runtime-captures/nmea_injector__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_VIEW_CONFIGURED_NMEA_SOCKETS,
    chains_from: None,
};

const ADD_EXTERNAL_NMEA_GPS_SOCKET: UserJourney = UserJourney {
    id: JourneyId::AddExternalNmeaGpsSocket,
    summary: Grounded::known(
        "Add a UDP or TCP socket so an external NMEA GPS device can inject positions as MAVLink",
        Provenance::doc(ADV, 539),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 533)),
    services: NMEA_INJECTOR_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::CreateNmeaSocket,
        "creation dialog submits socket kind, port, and MAVLink component ID to start listening",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Software(SoftwareRequirement::AdvancedMode),
            Provenance::source(NMEA_MENUS, 80),
        ),
        GroundedItem::new(
            Precondition::Hardware(HardwareRequirement::ExternalNmeaGps),
            Provenance::doc(ADV, 539),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the NMEA Injector page from the sidebar",
            None,
            Provenance::source(NMEA_MENUS, 77),
            None,
        ),
        operator_step(
            "Click the + button to open the new NMEA socket dialog",
            None,
            Provenance::source(NMEA_INJECTOR, 86),
            None,
        ),
        operator_step(
            "Choose socket kind (UDP or TCP), port, and MAVLink component ID",
            None,
            Provenance::doc(ADV, 543),
            None,
        ),
        operator_step(
            "Click Create to add the listening socket",
            Some(sourced_route(HttpMethod::Post, "/socks", Some("v1.0"), 48)),
            Provenance::source(NMEA_CREATE_DIALOG, 139),
            None,
        ),
    ]),
    availability: PRESENCE_ADD_EXTERNAL_NMEA_GPS_SOCKET,
    chains_from: None,
};

const REMOVE_CONFIGURED_NMEA_SOCKET: UserJourney = UserJourney {
    id: JourneyId::RemoveConfiguredNmeaSocket,
    summary: Grounded::known(
        "Remove a configured NMEA input socket from the NMEA Injector",
        Provenance::source(NMEA_SOCKET_CARD, 55),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 533)),
    services: NMEA_INJECTOR_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RemoveNmeaSocket,
        "socket card remove button deletes the matching kind, port, and component ID",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Software(SoftwareRequirement::AdvancedMode),
            Provenance::source(NMEA_MENUS, 80),
        ),
        GroundedItem::new(
            Precondition::Data(DataRequirement::NmeaSocketConfigured),
            Provenance::source(NMEA_INJECTOR, 71),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the NMEA Injector page from the sidebar",
            None,
            Provenance::source(NMEA_MENUS, 77),
            None,
        ),
        operator_step(
            "View the configured NMEA socket to remove",
            Some(sourced_route(HttpMethod::Get, "/socks", Some("v1.0"), 40)),
            Provenance::source(NMEA_INJECTOR, 138),
            Some(runtime_outcome(
                200,
                Some("[]"),
                "runtime-captures/nmea_injector__pi4_navigator_master.json#running_baseline",
            )),
        ),
        operator_step(
            "Click the remove button on the socket card",
            Some(sourced_route(
                HttpMethod::Delete,
                "/socks",
                Some("v1.0"),
                59,
            )),
            Provenance::source(NMEA_SOCKET_CARD, 55),
            Some(source_outcome(200, 65)),
        ),
    ]),
    availability: PRESENCE_REMOVE_CONFIGURED_NMEA_SOCKET,
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const NMEA_INJECTOR_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::NmeaInjector,
    Provenance::doc(ADV, 536),
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
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(NMEA_MAIN, line),
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

const fn source_outcome(status: u16, line: u32) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            transition: None,
        },
        Provenance::source(NMEA_MAIN, line),
    )
}
