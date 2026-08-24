use crate::journey_presence::{
    PRESENCE_BROWSE_AVAILABLE_WEB_SERVICES, PRESENCE_MONITOR_INTERNET_CONNECTIVITY,
    PRESENCE_PROBE_INTERFACE_INTERNET_CONNECTIVITY, PRESENCE_VERIFY_INTERNET_CONNECTIVITY,
};
use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, BodyKind, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef,
    SoftwareAssumption, StepOutcome, UseCase, Visibility,
};

const ADV: &str = "content/usage/advanced/index.md";
const GETTING: &str = "content/usage/getting-started/index.md";
const DEV_CORE: &str = "content/development/core/index.md";
const HELPER_MAIN: &str = "core/services/helper/main.py";
const HELPER_STORE: &str = "core/frontend/src/store/helper.ts";
const HELPER_MENUS: &str = "core/frontend/src/menus.ts";
const NETWORK_PRIORITY: &str = "core/frontend/src/components/app/NetworkInterfacePriorityMenu.vue";
const REQUIRE_INTERNET: &str = "core/frontend/src/components/wizard/RequireInternet.vue";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const JOURNEYS: &[UseCase] = &[
    MONITOR_INTERNET_CONNECTIVITY,
    VERIFY_INTERNET_CONNECTIVITY,
    BROWSE_AVAILABLE_WEB_SERVICES,
    PROBE_INTERFACE_INTERNET_CONNECTIVITY,
];

const MONITOR_INTERNET_CONNECTIVITY: UseCase = UseCase {
    id: JourneyId::MonitorInternetConnectivity,
    summary: Grounded::known(
        "See whether the vehicle is connected to the internet",
        Provenance::doc(ADV, 141, "- See whether the vehicle is connected to the internet (upda"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 139, "##### Internet Status and Management")),
    services: HELPER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::CheckInternetConnectivity,
        "header internet indicator reflects reachability of probe websites",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "View the internet connectivity indicator in the BlueOS header",
            None,
            Provenance::doc(ADV, 141, "- See whether the vehicle is connected to the internet (upda"),
            None,
        ),
        service_step(
            "Poll configured websites to refresh internet connectivity state (every 20 seconds)",
            Some(sourced_route(HttpMethod::Get, "/check_internet_access", Some("v1.0"), 540, "def check_internet_a")),
            Provenance::source(HELPER_STORE, 45, "{ delay: 20000 },"),
            Some(runtime_outcome(
                200,
                Some("\"online\": true"),
                BodyKind::Payload,
                "runtime-captures/helper__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_MONITOR_INTERNET_CONNECTIVITY,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "Polls /check_internet_access to refresh the header indicator without mutating network or vehicle settings",
        ),
    ),
    chains_from: None,
};

const VERIFY_INTERNET_CONNECTIVITY: UseCase = UseCase {
    id: JourneyId::VerifyInternetConnectivity,
    summary: Grounded::known(
        "Confirm the BlueOS header shows internet connectivity after network setup",
        Provenance::doc(GETTING, 100, "[show that it has internet connectivity](../advanced/#intern"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(GETTING, 74, "### Connect Internet")),
    services: HELPER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::CheckInternetConnectivity,
        "setup flows confirm probe websites are reachable before continuing",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Network(NetworkState::Online),
        Provenance::doc(GETTING, 76, "When starting out, it's important to connect your vehicle to"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Check that the BlueOS header shows internet connectivity",
            None,
            Provenance::doc(GETTING, 100, "[show that it has internet connectivity](../advanced/#intern"),
            None,
        ),
        operator_step(
            "Run the internet connectivity check used by the setup wizard",
            Some(sourced_route(HttpMethod::Get, "/check_internet_access", Some("v1.0"), 540, "def check_internet_a")),
            Provenance::source(REQUIRE_INTERNET, 94, "url: '/helper/latest/check_internet_access',"),
            Some(runtime_outcome(
                200,
                Some("\"online\": true"),
                BodyKind::Payload,
                "runtime-captures/helper__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_VERIFY_INTERNET_CONNECTIVITY,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "Setup wizard confirms header connectivity via read-only probe without changing vehicle configuration",
        ),
    ),
    chains_from: None,
};

const BROWSE_AVAILABLE_WEB_SERVICES: UseCase = UseCase {
    id: JourneyId::BrowseAvailableWebServices,
    summary: Grounded::known(
        "Browse HTTP services running on BlueOS with ports, names, and API documentation links",
        Provenance::doc(
            ADV,
            360,
            "The Available Services page provides developer access to the",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 357, "{% pirate() %}"),
    ),
    services: HELPER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DiscoverWebServices,
        "Available Services page lists scanned HTTP servers and swagger endpoints",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Available Services page from the sidebar",
            None,
            Provenance::source(HELPER_MENUS, 17, "title: 'Available Services',"),
            None,
        ),
        operator_step(
            "View scanned services with port, name, webpage, API documentation, and versions",
            Some(sourced_route(
                HttpMethod::Get,
                "/web_services",
                Some("v1.0"),
                529,
                "def web_services() -> Any:",
            )),
            Provenance::doc(ADV, 363, "- the port it is served at"),
            Some(runtime_outcome(
                200,
                Some("\"valid\": true"),
                BodyKind::Payload,
                "runtime-captures/helper__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_BROWSE_AVAILABLE_WEB_SERVICES,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "GET /web_services lists scanned HTTP endpoints without changing service state",
        ),
    ),
    chains_from: None,
};

const PROBE_INTERFACE_INTERNET_CONNECTIVITY: UseCase = UseCase {
    id: JourneyId::ProbeInterfaceInternetConnectivity,
    summary: Grounded::known(
        "Display internet availability on each network interface while configuring priority",
        Provenance::doc(ADV, 147, "- Displays internet availability on each network"),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 144, "{% pirate() %}")),
    services: HELPER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ProbeInterfaceConnectivity,
        "network priority menu pings a reachable host through each interface",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareAssumption::PirateMode),
        Provenance::doc(ADV, 144, "{% pirate() %}"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the network interface priority menu from the internet tray",
            None,
            Provenance::doc(ADV, 145, "- Configure network priority ordering"),
            None,
        ),
        operator_step(
            "View per-interface internet availability while reordering interfaces",
            Some(sourced_route(HttpMethod::Get, "/ping?host=1.1.1.1", Some("v1.0"), 583, "async def ping(host: str")),
            Provenance::source(NETWORK_PRIORITY, 120, "const result = await helper.ping({ host, iface: iface.name }"),
            Some(runtime_outcome(
                200,
                Some("true"),
                BodyKind::Payload,
                "runtime-captures/helper__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_PROBE_INTERFACE_INTERNET_CONNECTIVITY,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "GET /ping probes per-interface reachability without altering interface priority or addresses",
        ),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const HELPER_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Helper,
    Provenance::doc(
        DEV_CORE,
        76,
        "| [helper](https://github.com/bluerobotics/BlueOS/tree/maste",
    ),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Helper,
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
        Provenance::source(HELPER_MAIN, line, anchor),
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
            actor: Actor::Service(ServiceId::Helper),
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
