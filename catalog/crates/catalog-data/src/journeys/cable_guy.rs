use crate::journey_presence::{
    PRESENCE_ACQUIRE_DYNAMIC_IP_ADDRESS, PRESENCE_ASSIGN_STATIC_IP_ADDRESS,
    PRESENCE_CONFIGURE_HOST_DNS, PRESENCE_DISABLE_ONBOARD_DHCP_SERVER,
    PRESENCE_ENABLE_ONBOARD_DHCP_SERVER, PRESENCE_SET_NETWORK_INTERFACE_PRIORITY,
};
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, BodyKind, DataAssumption, HttpMethod, JourneyStep, Precondition, RouteRef,
    SoftwareAssumption, StepOutcome, UseCase, Visibility,
};

const ADV: &str = "content/usage/advanced/index.md";
const CABLE_GUY_MAIN: &str = "core/services/cable_guy/main.py";
const ADDRESS_DIALOG: &str = "core/frontend/src/components/ethernet/AddressCreationDialog.vue";
const DHCP_DIALOG: &str = "core/frontend/src/components/ethernet/DHCPServerDialog.vue";
const DNS_MENU: &str = "core/frontend/src/components/app/DnsConfigurationMenu.vue";
const INTERFACE_CARD: &str = "core/frontend/src/components/ethernet/InterfaceCard.vue";
const NETWORK_MENU: &str = "core/frontend/src/components/app/NetworkInterfaceMenu.vue";
const NETWORK_PRIORITY: &str = "core/frontend/src/components/app/NetworkInterfacePriorityMenu.vue";

pub const JOURNEYS: &[UseCase] = &[
    ASSIGN_STATIC_IP_ADDRESS,
    ACQUIRE_DYNAMIC_IP_ADDRESS,
    ENABLE_ONBOARD_DHCP_SERVER,
    DISABLE_ONBOARD_DHCP_SERVER,
    SET_NETWORK_INTERFACE_PRIORITY,
    CONFIGURE_HOST_DNS,
];

const ASSIGN_STATIC_IP_ADDRESS: UseCase = UseCase {
    id: JourneyId::AssignStaticIpAddress,
    summary: Grounded::known(
        "Assign a static IP address to a wired ethernet or USB-OTG interface",
        Provenance::doc(ADV, 99, "- A static IP"),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(
            ADV,
            98,
            "When configuring a wired interface, choose between:",
        ),
    ),
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
            Provenance::doc(
                ADV,
                98,
                "When configuring a wired interface, choose between:",
            ),
            None,
        ),
        operator_step(
            "Click Add static IP on the target wired interface",
            None,
            Provenance::source(INTERFACE_CARD, 44, "Add <br> static IP"),
            None,
        ),
        operator_step(
            "Enter the IP address and create the static assignment",
            Some(sourced_route(
                HttpMethod::Post,
                "/address",
                Some("v1.0"),
                69,
                "@app.post(\"/address\", summary=\"Add IP address to interface.\"",
            )),
            Provenance::source(
                ADDRESS_DIALOG,
                87,
                "await ethernet.addAddress({ interface_name: this.interfaceNa",
            ),
            Some(source_outcome(
                200,
                71,
                "def add_address(interface_name: str, ip_address: str) -> Any",
            )),
        ),
    ]),
    availability: PRESENCE_ASSIGN_STATIC_IP_ADDRESS,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "Applying a static address to a live interface can move or break the management route",
        ),
    ),
    chains_from: None,
};

const ACQUIRE_DYNAMIC_IP_ADDRESS: UseCase = UseCase {
    id: JourneyId::AcquireDynamicIpAddress,
    summary: Grounded::known(
        "Request a dynamic IP address on a wired ethernet or USB-OTG interface",
        Provenance::doc(ADV, 100, "- A dynamic IP"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 98, "When configuring a wired interface,")),
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
            Provenance::doc(ADV, 98, "When configuring a wired interface, choose between:"),
            None,
        ),
        operator_step(
            "Click Ask for dynamic IP on the target wired interface",
            Some(sourced_route(HttpMethod::Post, "/dynamic_ip", Some("v1.0"), 115, "@app.post(\"/dynamic_ip\", summa")),
            Provenance::source(INTERFACE_CARD, 324, "await ethernet.triggerDynamicIP(this.adapter.name)"),
            Some(source_outcome(200, 117, "def trigger_dynamic_ip_acquisition(interface_name: str) -> A")),
        ),
    ]),
    availability: PRESENCE_ACQUIRE_DYNAMIC_IP_ADDRESS,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "DHCP client acquisition on an interface is undoable by removing the lease or restoring prior addressing",
        ),
    ),
    chains_from: None,
};

const ENABLE_ONBOARD_DHCP_SERVER: UseCase = UseCase {
    id: JourneyId::EnableOnboardDhcpServer,
    summary: Grounded::known(
        "Enable the onboard DHCP server on a wired interface, optionally as a backup server",
        Provenance::doc(ADV, 101, "- A DHCP server"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 98, "When configuring a wired interface,")),
    services: CABLE_GUY_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::EnableDhcpServer,
        "ethernet tray starts a local DHCP server on the interface gateway address",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other(
            "The interface has at least one static IP address to use as the DHCP gateway",
        ),
        Provenance::source(INTERFACE_CARD, 72, "v-tooltip=\"!is_static_ip_present ? 'A static IP address is r"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the ethernet tray menu from the header bar",
            None,
            Provenance::doc(ADV, 98, "When configuring a wired interface, choose between:"),
            None,
        ),
        operator_step(
            "Click Enable DHCP server on an interface that already has a static IP",
            None,
            Provenance::source(INTERFACE_CARD, 80, "Enable <br> DHCP server"),
            None,
        ),
        operator_step(
            "Select the server gateway, optionally enable backup mode, and confirm",
            Some(sourced_route(HttpMethod::Post, "/dhcp", Some("v1.0"), 99, "@app.post(\"/dhcp\", summary=\"Add loca")),
            Provenance::source(DHCP_DIALOG, 141, "await ethernet.addDHCPServer({"),
            Some(source_outcome(200, 101, "async def add_dhcp_server(interface_name: str, ipv4_gateway:")),
        ),
    ]),
    availability: PRESENCE_ENABLE_ONBOARD_DHCP_SERVER,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "Starting an onboard DHCP server affects addressing for every client on that link segment",
        ),
    ),
    chains_from: None,
};

const DISABLE_ONBOARD_DHCP_SERVER: UseCase = UseCase {
    id: JourneyId::DisableOnboardDhcpServer,
    summary: Grounded::known(
        "Disable the onboard DHCP server on a wired interface",
        Provenance::source(INTERFACE_CARD, 68, "Disable <br> DHCP server"),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(
            ADV,
            98,
            "When configuring a wired interface, choose between:",
        ),
    ),
    services: CABLE_GUY_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DisableDhcpServer,
        "ethernet tray removes the local DHCP server from the selected interface",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataAssumption::OnboardDhcpServerActive),
        Provenance::source(INTERFACE_CARD, 63, "v-if=\"is_there_dhcp_server_already\""),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the ethernet tray menu from the header bar",
            None,
            Provenance::doc(
                ADV,
                98,
                "When configuring a wired interface, choose between:",
            ),
            None,
        ),
        operator_step(
            "Click Disable DHCP server on the target wired interface",
            Some(sourced_route(
                HttpMethod::Delete,
                "/dhcp",
                Some("v1.0"),
                107,
                "@app.delete(\"/dhcp\", summary=\"Remove local DHCP server from ",
            )),
            Provenance::source(
                INTERFACE_CARD,
                332,
                "await ethernet.RemoveDHCPServer(this.adapter.name)",
            ),
            Some(source_outcome(
                200,
                109,
                "def remove_dhcp_server(interface_name: str) -> Any:",
            )),
        ),
    ]),
    availability: PRESENCE_DISABLE_ONBOARD_DHCP_SERVER,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "Stopping the onboard DHCP server can be undone by re-enabling it on the same gateway",
        ),
    ),
    chains_from: None,
};

const SET_NETWORK_INTERFACE_PRIORITY: UseCase = UseCase {
    id: JourneyId::SetNetworkInterfacePriority,
    summary: Grounded::known(
        "Reorder network interfaces to choose which connection is preferred for internet access",
        Provenance::doc(ADV, 145, "- Configure network priority ordering"),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 144, "{% pirate() %}")),
    services: CABLE_GUY_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::SetInterfacePriority,
        "internet tray persists interface metric ordering used for default routes",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareAssumption::PirateMode),
        Provenance::doc(ADV, 144, "{% pirate() %}"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the internet indicator menu from the header bar",
            None,
            Provenance::doc(ADV, 145, "- Configure network priority ordering"),
            None,
        ),
        operator_step(
            "Open the Network Interface Priority tab",
            None,
            Provenance::source(
                NETWORK_MENU,
                54,
                "{ title: 'Network Interface Priority', icon: 'mdi-sort', value: 'network_interface_priority'",
            ),
            None,
        ),
        operator_step(
            "Drag interfaces into the desired priority order",
            None,
            Provenance::source(NETWORK_PRIORITY, 5, "Drag the network interfaces to move the highest priority to "),
            None,
        ),
        operator_step(
            "Apply the new interface priority ordering",
            Some(sourced_route(HttpMethod::Post, "/set_interfaces_priority", Some("v1.0"), 62, "@app.post(\"/set_in")),
            Provenance::source(NETWORK_PRIORITY, 141, "await ethernet.setInterfacesPriority(interface_priorities)"),
            Some(source_outcome(200, 64, "def set_interfaces_priority(interfaces: List[NetworkInterfac")),
        ),
    ]),
    availability: PRESENCE_SET_NETWORK_INTERFACE_PRIORITY,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "Reordering interface metrics can shift the default route off the link the operator uses",
        ),
    ),
    chains_from: None,
};

const CONFIGURE_HOST_DNS: UseCase = UseCase {
    id: JourneyId::ConfigureHostDns,
    summary: Grounded::known(
        "View and configure host DNS nameservers applied to /etc/resolv.conf",
        Provenance::doc(ADV, 153, "- View and configure DNS name servers"),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 152, "{% pirate() %}")),
    services: CABLE_GUY_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConfigureHostDns,
        "internet tray updates locked host nameserver entries via cable_guy",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareAssumption::PirateMode),
        Provenance::doc(ADV, 152, "{% pirate() %}"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the internet indicator menu from the header bar",
            None,
            Provenance::doc(ADV, 153, "- View and configure DNS name servers"),
            None,
        ),
        operator_step(
            "Switch to the Dns Configuration tab",
            None,
            Provenance::source(NETWORK_MENU, 55, "{ title: 'Dns Configuration', icon: 'mdi-dns', value: 'dns_c"),
            None,
        ),
        operator_step(
            "Edit nameserver entries and optional lock setting, then apply",
            Some(sourced_route(HttpMethod::Post, "/host_dns", Some("v1.0"), 130, "@app.post(\"/host_dns\", summary=")),
            Provenance::source(DNS_MENU, 171, "await ethernet.updateHostDNS({ host_nameservers: this.host_n"),
            Some(source_outcome(200, 132, "def update_host_dns(dns_data: DnsData) -> Any:")),
        ),
    ]),
    availability: PRESENCE_CONFIGURE_HOST_DNS,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "Persisting host nameserver changes alters resolver behavior for all outbound connections",
        ),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const CABLE_GUY_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::CableGuy,
    Provenance::doc(
        ADV,
        93,
        "{{ service(service=\"Cable Guy\", port=9090, link=\"/services/c",
    ),
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
    anchor: &'static str,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(CABLE_GUY_MAIN, line, anchor),
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

const fn source_outcome(status: u16, line: u32, anchor: &'static str) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind: BodyKind::Unknown,
            transition: None,
        },
        Provenance::source(CABLE_GUY_MAIN, line, anchor),
    )
}
