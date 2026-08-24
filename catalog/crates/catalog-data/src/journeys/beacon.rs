use crate::journey_presence::{
    PRESENCE_CHANGE_MDNS_HOSTNAME, PRESENCE_DISCOVER_BLUEOS_ON_NETWORK, PRESENCE_RENAME_VEHICLE,
};
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, BodyKind, HttpMethod, JourneyStep, NetworkResource, Precondition, RouteRef,
    StepOutcome, UseCase, Visibility,
};

const ADV: &str = "content/usage/advanced/index.md";
const BEACON_MAIN: &str = "core/services/beacon/main.py";
const DEV_CORE: &str = "content/development/core/index.md";
const GETTING: &str = "content/usage/getting-started/index.md";
const VEHICLE_BANNER: &str = "core/frontend/src/components/app/VehicleBanner.vue";

pub const JOURNEYS: &[UseCase] = &[
    RENAME_VEHICLE,
    CHANGE_MDNS_HOSTNAME,
    DISCOVER_BLUEOS_ON_NETWORK,
];

const RENAME_VEHICLE: UseCase =
    UseCase {
        id: JourneyId::RenameVehicle,
        summary: Grounded::known(
            "Set the vehicle name shown in the sidebar so it is easier to tell which vehicle you are connected to"
                ,
            Provenance::doc(ADV, 894, "- the vehicle name makes it easier to determine which vehicl"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875, "The vehicle identifier compone")),
        services: BEACON_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::SetVehicleName,
            "sidebar edit dialog persists the vehicle name via the beacon API",
        )]),
        preconditions: GroundedSet::known(&[]),
        steps: GroundedSet::known(&[
            operator_step(
                "Click edit on the vehicle identifier in the sidebar",
                None,
                Provenance::doc(ADV, 875, "The vehicle identifier components in the sidebar can be modi"),
                None,
            ),
            operator_step(
                "Enter a vehicle name and save",
                Some(sourced_route(HttpMethod::Post, "/vehicle_name", Some("v1.0"), 298, "@app.post(\"/vehicle_name")),
                Provenance::source(VEHICLE_BANNER, 160, "save_name() {"),
                Some(source_outcome(200, 298, "@app.post(\"/vehicle_name\", summary=\"Set the vehicle name\")")),
            ),
        ]),
        availability: PRESENCE_RENAME_VEHICLE,
        blast_radius: Grounded::known(
            BlastRadius::Reversible,
            Provenance::asserted(
                "Vehicle display name is a userdata label restored by posting the prior name",
            ),
        ),
        chains_from: None,
    };

const CHANGE_MDNS_HOSTNAME: UseCase = UseCase {
    id: JourneyId::ChangeMdnsHostname,
    summary: Grounded::known(
        "Change the mDNS hostname used to reach the BlueOS web interface in a browser",
        Provenance::doc(ADV, 895, "- changing the mDNS hostname changes the address you connect"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 875, "The vehicle identifier components ")),
    services: BEACON_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::SetMdnsHostname,
        "sidebar edit dialog updates the hostname broadcast for mDNS addresses",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Click edit on the vehicle identifier in the sidebar",
            None,
            Provenance::doc(ADV, 875, "The vehicle identifier components in the sidebar can be modi"),
            None,
        ),
        operator_step(
            "Enter an mDNS hostname and save",
            Some(sourced_route(HttpMethod::Post, "/hostname", Some("v1.0"), 286, "@app.post(\"/hostname\", summary=")),
            Provenance::source(VEHICLE_BANNER, 163, "save_mdns() {"),
            Some(source_outcome(200, 286, "@app.post(\"/hostname\", summary=\"Set the hostname for mDNS.\")")),
        ),
    ]),
    availability: PRESENCE_CHANGE_MDNS_HOSTNAME,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "Changing the mDNS hostname breaks the prior blueos.local bookmark until DNS-SD catches up",
        ),
    ),
    chains_from: None,
};

const DISCOVER_BLUEOS_ON_NETWORK: UseCase = UseCase {
    id: JourneyId::DiscoverBlueosOnNetwork,
    summary: Grounded::known(
        "Open the BlueOS web interface at blueos.local on the local network",
        Provenance::doc(GETTING, 29, "- When BlueOS is connected via a wired connection, it is als"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(GETTING, 26, "### Interface Access")),
    services: BEACON_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::AdvertiseMdnsDomains,
        "beacon publishes mDNS records that make blueos.local resolvable on the LAN",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::NetworkResource(NetworkResource::WiredEthernetPresent),
            Provenance::doc(GETTING, 29, "- When BlueOS is connected via a wired connection, it is als"),
        ),
        GroundedItem::new(
            Precondition::Other(
                "BlueOS is connected via a wired connection so blueos.local is reachable",
            ),
            Provenance::doc(GETTING, 29, "- When BlueOS is connected via a wired connection, it is als"),
        ),
    ]),
    steps: GroundedSet::known(&[
        service_step(
            "Publish mDNS domain advertisements on available network interfaces",
            None,
            Provenance::source(BEACON_MAIN, 230, "async def run(self) -> None:"),
            None,
        ),
        operator_step(
            "Open http://blueos.local in a web browser",
            None,
            Provenance::doc(GETTING, 29, "- When BlueOS is connected via a wired connection, it is als"),
            None,
        ),
    ]),
    availability: PRESENCE_DISCOVER_BLUEOS_ON_NETWORK,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "Operator opens the web UI via existing mDNS advertisement without mutating system state",
        ),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const BEACON_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Beacon,
    Provenance::doc(
        DEV_CORE,
        70,
        "| [Beacon Service](https://github.com/bluerobotics/BlueOS/tr",
    ),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Beacon,
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
        Provenance::source(BEACON_MAIN, line, anchor),
    )
}

const fn source_outcome(status: u16, line: u32, anchor: &'static str) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind: BodyKind::Unknown,
            transition: None,
        },
        Provenance::source(BEACON_MAIN, line, anchor),
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
            actor: Actor::Service(ServiceId::Beacon),
            description,
            route,
            outcome,
        },
        provenance,
    )
}
