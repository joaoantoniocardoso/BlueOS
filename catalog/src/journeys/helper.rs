use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef, SoftwareRequirement,
    StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const GETTING: &str = "content/usage/getting-started/index.md";
const DEV_CORE: &str = "content/development/core/index.md";
const HELPER_MAIN: &str = "core/services/helper/main.py";
const HELPER_STORE: &str = "core/frontend/src/store/helper.ts";
const HELPER_MENUS: &str = "core/frontend/src/menus.ts";
const NETWORK_PRIORITY: &str = "core/frontend/src/components/app/NetworkInterfacePriorityMenu.vue";
const REQUIRE_INTERNET: &str = "core/frontend/src/components/wizard/RequireInternet.vue";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const JOURNEYS: &[UserJourney] = &[
    MONITOR_INTERNET_CONNECTIVITY,
    VERIFY_INTERNET_CONNECTIVITY,
    BROWSE_AVAILABLE_WEB_SERVICES,
    PROBE_INTERFACE_INTERNET_CONNECTIVITY,
];

const MONITOR_INTERNET_CONNECTIVITY: UserJourney = UserJourney {
    id: JourneyId::MonitorInternetConnectivity,
    summary: Grounded::known(
        "See whether the vehicle is connected to the internet",
        Provenance::doc(ADV, 141),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 139)),
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
            Provenance::doc(ADV, 141),
            None,
        ),
        service_step(
            "Poll configured websites to refresh internet connectivity state (every 20 seconds)",
            Some(sourced_route(
                HttpMethod::Get,
                "/check_internet_access",
                Some("v1.0"),
                540,
            )),
            Provenance::source(HELPER_STORE, 45),
            Some(runtime_outcome(
                200,
                Some("\"online\": true"),
                "runtime-captures/helper__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    chains_from: None,
};

const VERIFY_INTERNET_CONNECTIVITY: UserJourney = UserJourney {
    id: JourneyId::VerifyInternetConnectivity,
    summary: Grounded::known(
        "Confirm the BlueOS header shows internet connectivity after network setup",
        Provenance::doc(GETTING, 100),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(GETTING, 74)),
    services: HELPER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::CheckInternetConnectivity,
        "setup flows confirm probe websites are reachable before continuing",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Network(NetworkState::Online),
        Provenance::doc(GETTING, 76),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Check that the BlueOS header shows internet connectivity",
            None,
            Provenance::doc(GETTING, 100),
            None,
        ),
        operator_step(
            "Run the internet connectivity check used by the setup wizard",
            Some(sourced_route(
                HttpMethod::Get,
                "/check_internet_access",
                Some("v1.0"),
                540,
            )),
            Provenance::source(REQUIRE_INTERNET, 94),
            Some(runtime_outcome(
                200,
                Some("\"online\": true"),
                "runtime-captures/helper__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    chains_from: None,
};

const BROWSE_AVAILABLE_WEB_SERVICES: UserJourney = UserJourney {
    id: JourneyId::BrowseAvailableWebServices,
    summary: Grounded::known(
        "Browse HTTP services running on BlueOS with ports, names, and API documentation links",
        Provenance::doc(ADV, 360),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 357)),
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
            Provenance::source(HELPER_MENUS, 17),
            None,
        ),
        operator_step(
            "View scanned services with port, name, webpage, API documentation, and versions",
            Some(sourced_route(
                HttpMethod::Get,
                "/web_services",
                Some("v1.0"),
                529,
            )),
            Provenance::doc(ADV, 363),
            Some(runtime_outcome(
                200,
                Some("\"valid\": true"),
                "runtime-captures/helper__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    chains_from: None,
};

const PROBE_INTERFACE_INTERNET_CONNECTIVITY: UserJourney = UserJourney {
    id: JourneyId::ProbeInterfaceInternetConnectivity,
    summary: Grounded::known(
        "Display internet availability on each network interface while configuring priority",
        Provenance::doc(ADV, 147),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 144)),
    services: HELPER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ProbeInterfaceConnectivity,
        "network priority menu pings a reachable host through each interface",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareRequirement::PirateMode),
        Provenance::doc(ADV, 144),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the network interface priority menu from the internet tray",
            None,
            Provenance::doc(ADV, 145),
            None,
        ),
        operator_step(
            "View per-interface internet availability while reordering interfaces",
            Some(sourced_route(
                HttpMethod::Get,
                "/ping?host=1.1.1.1",
                Some("v1.0"),
                583,
            )),
            Provenance::source(NETWORK_PRIORITY, 120),
            Some(runtime_outcome(
                200,
                Some("true"),
                "runtime-captures/helper__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const HELPER_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Helper,
    Provenance::doc(DEV_CORE, 76),
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
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(HELPER_MAIN, line),
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
