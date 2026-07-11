use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const BEACON_MAIN: &str = "core/services/beacon/main.py";
const DEV_CORE: &str = "content/development/core/index.md";
const GETTING: &str = "content/usage/getting-started/index.md";
const VEHICLE_BANNER: &str = "core/frontend/src/components/app/VehicleBanner.vue";

pub fn journeys() -> Vec<UserJourney> {
    vec![
        rename_vehicle(),
        change_mdns_hostname(),
        discover_blueos_on_network(),
    ]
}

fn rename_vehicle() -> UserJourney {
    UserJourney {
        id: JourneyId::RenameVehicle,
        summary: Grounded::known(
            "Set the vehicle name shown in the sidebar so it is easier to tell which vehicle you are connected to"
                .into(),
            Provenance::doc(ADV, 894),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875)),
        services: beacon_services(),
        capability_refs: GroundedSet::known(vec![cap(CapabilityId::SetVehicleName,
            "sidebar edit dialog persists the vehicle name via the beacon API",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Click edit on the vehicle identifier in the sidebar",
                None,
                Provenance::doc(ADV, 875),
                None,
            ),
            operator_step(
                "Enter a vehicle name and save",
                Some(sourced_route(HttpMethod::Post, "/vehicle_name", Some("v1.0"), 298)),
                Provenance::source(VEHICLE_BANNER, 160),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn change_mdns_hostname() -> UserJourney {
    UserJourney {
        id: JourneyId::ChangeMdnsHostname,
        summary: Grounded::known(
            "Change the mDNS hostname used to reach the BlueOS web interface in a browser".into(),
            Provenance::doc(ADV, 895),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875)),
        services: beacon_services(),
        capability_refs: GroundedSet::known(vec![cap(
            CapabilityId::SetMdnsHostname,
            "sidebar edit dialog updates the hostname broadcast for mDNS addresses",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Click edit on the vehicle identifier in the sidebar",
                None,
                Provenance::doc(ADV, 875),
                None,
            ),
            operator_step(
                "Enter an mDNS hostname and save",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/hostname",
                    Some("v1.0"),
                    286,
                )),
                Provenance::source(VEHICLE_BANNER, 163),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn discover_blueos_on_network() -> UserJourney {
    UserJourney {
        id: JourneyId::DiscoverBlueosOnNetwork,
        summary: Grounded::known(
            "Open the BlueOS web interface at blueos.local on the local network".into(),
            Provenance::doc(GETTING, 29),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(GETTING, 26)),
        services: beacon_services(),
        capability_refs: GroundedSet::known(vec![cap(
            CapabilityId::AdvertiseMdnsDomains,
            "beacon publishes mDNS records that make blueos.local resolvable on the LAN",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other(
                "BlueOS is connected via a wired connection so blueos.local is reachable".into(),
            ),
            Provenance::doc(GETTING, 29),
        )]),
        steps: GroundedSet::known(vec![
            service_step(
                "Publish mDNS domain advertisements on available network interfaces",
                None,
                Provenance::source(BEACON_MAIN, 230),
                None,
            ),
            operator_step(
                "Open http://blueos.local in a web browser",
                None,
                Provenance::doc(GETTING, 29),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: CapabilityId, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

fn beacon_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::Beacon,
        Provenance::doc(DEV_CORE, 70),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Beacon,
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
        Provenance::source(BEACON_MAIN, line),
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
            actor: Actor::Service(ServiceId::Beacon),
            description: description.into(),
            route,
            outcome,
        },
        provenance,
    )
}
