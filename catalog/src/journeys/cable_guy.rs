use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const CABLE_GUY_MAIN: &str = "core/services/cable_guy/main.py";
const ADDRESS_DIALOG: &str = "core/frontend/src/components/ethernet/AddressCreationDialog.vue";
const DHCP_DIALOG: &str = "core/frontend/src/components/ethernet/DHCPServerDialog.vue";
const DNS_MENU: &str = "core/frontend/src/components/app/DnsConfigurationMenu.vue";
const INTERFACE_CARD: &str = "core/frontend/src/components/ethernet/InterfaceCard.vue";
const NETWORK_MENU: &str = "core/frontend/src/components/app/NetworkInterfaceMenu.vue";
const NETWORK_PRIORITY: &str = "core/frontend/src/components/app/NetworkInterfacePriorityMenu.vue";

pub const JOURNEYS: &[UserJourney] = &[
    ASSIGN_STATIC_IP_ADDRESS,
    ACQUIRE_DYNAMIC_IP_ADDRESS,
    ENABLE_ONBOARD_DHCP_SERVER,
    DISABLE_ONBOARD_DHCP_SERVER,
    SET_NETWORK_INTERFACE_PRIORITY,
    CONFIGURE_HOST_DNS,
];

const ASSIGN_STATIC_IP_ADDRESS: UserJourney = UserJourney {
    id: JourneyId::AssignStaticIpAddress,
    summary: Grounded::known(
        "Assign a static IP address to a wired ethernet or USB-OTG interface",
        Provenance::doc(ADV, 99),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 98)),
    services: CABLE_GUY_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::AssignStaticIp,
        "ethernet tray adds a static IPv4 address to the selected interface",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the ethernet tray menu from the header bar",
            None,
            Provenance::doc(ADV, 98),
            None,
        ),
        operator_step(
            "Click Add static IP on the target wired interface",
            None,
            Provenance::source(INTERFACE_CARD, 44),
            None,
        ),
        operator_step(
            "Enter the IP address and create the static assignment",
            Some(sourced_route(
                HttpMethod::Post,
                "/address",
                Some("v1.0"),
                69,
            )),
            Provenance::source(ADDRESS_DIALOG, 87),
            None,
        ),
    ]),
    chains_from: None,
};

const ACQUIRE_DYNAMIC_IP_ADDRESS: UserJourney = UserJourney {
    id: JourneyId::AcquireDynamicIpAddress,
    summary: Grounded::known(
        "Request a dynamic IP address on a wired ethernet or USB-OTG interface",
        Provenance::doc(ADV, 100),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 98)),
    services: CABLE_GUY_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::AcquireDynamicIp,
        "ethernet tray triggers DHCP client acquisition on the selected interface",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the ethernet tray menu from the header bar",
            None,
            Provenance::doc(ADV, 98),
            None,
        ),
        operator_step(
            "Click Ask for dynamic IP on the target wired interface",
            Some(sourced_route(
                HttpMethod::Post,
                "/dynamic_ip",
                Some("v1.0"),
                115,
            )),
            Provenance::source(INTERFACE_CARD, 324),
            None,
        ),
    ]),
    chains_from: None,
};

const ENABLE_ONBOARD_DHCP_SERVER: UserJourney = UserJourney {
    id: JourneyId::EnableOnboardDhcpServer,
    summary: Grounded::known(
        "Enable the onboard DHCP server on a wired interface, optionally as a backup server",
        Provenance::doc(ADV, 101),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 98)),
    services: CABLE_GUY_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::EnableDhcpServer,
        "ethernet tray starts a local DHCP server on the interface gateway address",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other(
            "The interface has at least one static IP address to use as the DHCP gateway",
        ),
        Provenance::source(INTERFACE_CARD, 72),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the ethernet tray menu from the header bar",
            None,
            Provenance::doc(ADV, 98),
            None,
        ),
        operator_step(
            "Click Enable DHCP server on an interface that already has a static IP",
            None,
            Provenance::source(INTERFACE_CARD, 80),
            None,
        ),
        operator_step(
            "Select the server gateway, optionally enable backup mode, and confirm",
            Some(sourced_route(HttpMethod::Post, "/dhcp", Some("v1.0"), 99)),
            Provenance::source(DHCP_DIALOG, 141),
            None,
        ),
    ]),
    chains_from: None,
};

const DISABLE_ONBOARD_DHCP_SERVER: UserJourney = UserJourney {
    id: JourneyId::DisableOnboardDhcpServer,
    summary: Grounded::known(
        "Disable the onboard DHCP server on a wired interface",
        Provenance::source(INTERFACE_CARD, 68),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 98)),
    services: CABLE_GUY_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DisableDhcpServer,
        "ethernet tray removes the local DHCP server from the selected interface",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("The interface has an active onboard DHCP server"),
        Provenance::source(INTERFACE_CARD, 63),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the ethernet tray menu from the header bar",
            None,
            Provenance::doc(ADV, 98),
            None,
        ),
        operator_step(
            "Click Disable DHCP server on the target wired interface",
            Some(sourced_route(
                HttpMethod::Delete,
                "/dhcp",
                Some("v1.0"),
                107,
            )),
            Provenance::source(INTERFACE_CARD, 332),
            None,
        ),
    ]),
    chains_from: None,
};

const SET_NETWORK_INTERFACE_PRIORITY: UserJourney = UserJourney {
    id: JourneyId::SetNetworkInterfacePriority,
    summary: Grounded::known(
        "Reorder network interfaces to choose which connection is preferred for internet access",
        Provenance::doc(ADV, 145),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 144)),
    services: CABLE_GUY_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::SetInterfacePriority,
        "internet tray persists interface metric ordering used for default routes",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("Pirate mode enabled to access network interface management"),
        Provenance::doc(ADV, 144),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the internet indicator menu from the header bar",
            None,
            Provenance::doc(ADV, 145),
            None,
        ),
        operator_step(
            "Open the Network Interface Priority tab",
            None,
            Provenance::source(NETWORK_MENU, 54),
            None,
        ),
        operator_step(
            "Drag interfaces into the desired priority order",
            None,
            Provenance::source(NETWORK_PRIORITY, 5),
            None,
        ),
        operator_step(
            "Apply the new interface priority ordering",
            Some(sourced_route(
                HttpMethod::Post,
                "/set_interfaces_priority",
                Some("v1.0"),
                62,
            )),
            Provenance::source(NETWORK_PRIORITY, 141),
            None,
        ),
    ]),
    chains_from: None,
};

const CONFIGURE_HOST_DNS: UserJourney = UserJourney {
    id: JourneyId::ConfigureHostDns,
    summary: Grounded::known(
        "View and configure host DNS nameservers applied to /etc/resolv.conf",
        Provenance::doc(ADV, 153),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 152)),
    services: CABLE_GUY_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConfigureHostDns,
        "internet tray updates locked host nameserver entries via cable_guy",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("Pirate mode enabled to access DNS configuration"),
        Provenance::doc(ADV, 152),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the internet indicator menu from the header bar",
            None,
            Provenance::doc(ADV, 153),
            None,
        ),
        operator_step(
            "Switch to the Dns Configuration tab",
            None,
            Provenance::source(NETWORK_MENU, 55),
            None,
        ),
        operator_step(
            "Edit nameserver entries and optional lock setting, then apply",
            Some(sourced_route(
                HttpMethod::Post,
                "/host_dns",
                Some("v1.0"),
                130,
            )),
            Provenance::source(DNS_MENU, 171),
            None,
        ),
    ]),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const CABLE_GUY_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::CableGuy,
    Provenance::doc(ADV, 93),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::CableGuy,
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
        Provenance::source(CABLE_GUY_MAIN, line),
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
