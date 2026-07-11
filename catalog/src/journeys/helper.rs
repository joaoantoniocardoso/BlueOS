use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef, UserJourney, Visibility,
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

#[allow(dead_code)]
pub fn journeys() -> Vec<UserJourney> {
    vec![
        monitor_internet_connectivity(),
        verify_internet_connectivity(),
        browse_available_web_services(),
        probe_interface_internet_connectivity(),
    ]
}

fn monitor_internet_connectivity() -> UserJourney {
    UserJourney {
        id: JourneyId("monitor_internet_connectivity".into()),
        summary: Grounded::known(
            "See whether the vehicle is connected to the internet".into(),
            Provenance::doc(ADV, 141),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 139)),
        services: helper_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "check_internet_connectivity",
            "header internet indicator reflects reachability of probe websites",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "View the internet connectivity indicator in the BlueOS header",
                None,
                Provenance::doc(ADV, 141),
            ),
            service_step(
                "Poll configured websites to refresh internet connectivity state (every 20 seconds)",
                Some(sourced_route(HttpMethod::Get, "/check_internet_access", Some("v1.0"), 540)),
                Provenance::source(HELPER_STORE, 45),
            ),
        ]),
        chains_from: None,
    }
}

fn verify_internet_connectivity() -> UserJourney {
    UserJourney {
        id: JourneyId("verify_internet_connectivity".into()),
        summary: Grounded::known(
            "Confirm the BlueOS header shows internet connectivity after network setup".into(),
            Provenance::doc(GETTING, 100),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(GETTING, 74)),
        services: helper_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "check_internet_connectivity",
            "setup flows confirm probe websites are reachable before continuing",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(GETTING, 76),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Check that the BlueOS header shows internet connectivity",
                None,
                Provenance::doc(GETTING, 100),
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
            ),
        ]),
        chains_from: None,
    }
}

fn browse_available_web_services() -> UserJourney {
    UserJourney {
        id: JourneyId("browse_available_web_services".into()),
        summary: Grounded::known(
            "Browse HTTP services running on BlueOS with ports, names, and API documentation links"
                .into(),
            Provenance::doc(ADV, 360),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 357)),
        services: helper_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "discover_web_services",
            "Available Services page lists scanned HTTP servers and swagger endpoints",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Available Services page from the sidebar",
                None,
                Provenance::source(HELPER_MENUS, 17),
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
            ),
        ]),
        chains_from: None,
    }
}

fn probe_interface_internet_connectivity() -> UserJourney {
    UserJourney {
        id: JourneyId("probe_interface_internet_connectivity".into()),
        summary: Grounded::known(
            "Display internet availability on each network interface while configuring priority"
                .into(),
            Provenance::doc(ADV, 147),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 144)),
        services: helper_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "probe_interface_connectivity",
            "network priority menu pings a reachable host through each interface",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other(
                "Pirate mode enabled to access network interface management".into(),
            ),
            Provenance::doc(ADV, 161),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the network interface priority menu from the internet tray",
                None,
                Provenance::doc(ADV, 145),
            ),
            operator_step(
                "View per-interface internet availability while reordering interfaces",
                Some(sourced_route(HttpMethod::Get, "/ping", Some("v1.0"), 583)),
                Provenance::source(NETWORK_PRIORITY, 120),
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn helper_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId("helper".into()),
        Provenance::doc(DEV_CORE, 76),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId("helper".into()),
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
        Provenance::source(HELPER_MAIN, line),
    )
}

fn operator_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description: description.into(),
            route,
            outcome: None,
        },
        provenance,
    )
}

fn service_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Service(ServiceId("helper".into())),
            description: description.into(),
            route,
            outcome: None,
        },
        provenance,
    )
}
