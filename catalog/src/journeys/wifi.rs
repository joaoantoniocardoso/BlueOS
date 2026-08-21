use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, BodyKind, DataRequirement, HttpMethod, JourneyStep, NetworkResource,
    Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::journey_presence::{
    PRESENCE_AUTOCONNECT_TO_SAVED_WIFI_NETWORK, PRESENCE_CONFIGURE_HOTSPOT_CREDENTIALS,
    PRESENCE_CONNECT_TO_HIDDEN_WIFI_NETWORK, PRESENCE_CONNECT_TO_WIFI_NETWORK,
    PRESENCE_DETECT_WIFI_AP_LOSS, PRESENCE_DISCONNECT_FROM_WIFI_NETWORK,
    PRESENCE_FORCE_WIFI_NETWORK_PASSWORD, PRESENCE_FORGET_SAVED_WIFI_NETWORK,
    PRESENCE_RECONNECT_TO_SAVED_WIFI_NETWORK, PRESENCE_REJECT_INVALID_WIFI_CREDENTIALS,
    PRESENCE_TOGGLE_HOTSPOT, PRESENCE_TOGGLE_SMART_HOTSPOT,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const GETTING: &str = "content/usage/getting-started/index.md";
const WIFI_MAIN: &str = "core/services/wifi/main.py";
const CONNECTION_DIALOG: &str = "core/frontend/src/components/wifi/ConnectionDialog.vue";
const DISCONNECTION_DIALOG: &str = "core/frontend/src/components/wifi/DisconnectionDialog.vue";
const WIFI_MANAGER: &str = "core/frontend/src/components/wifi/WifiManager.vue";
const WIFI_SETTINGS: &str = "core/frontend/src/components/wifi/WifiSettingsDialog.vue";
const WIFI_TRAY: &str = "core/frontend/src/components/wifi/WifiTrayMenu.vue";

pub const JOURNEYS: &[UserJourney] = &[
    CONNECT_TO_WIFI_NETWORK,
    CONNECT_TO_HIDDEN_WIFI_NETWORK,
    DISCONNECT_FROM_WIFI_NETWORK,
    FORGET_SAVED_WIFI_NETWORK,
    FORCE_WIFI_NETWORK_PASSWORD,
    RECONNECT_TO_SAVED_WIFI_NETWORK,
    REJECT_INVALID_WIFI_CREDENTIALS,
    DETECT_WIFI_AP_LOSS,
    AUTOCONNECT_TO_SAVED_WIFI_NETWORK,
    TOGGLE_HOTSPOT,
    CONFIGURE_HOTSPOT_CREDENTIALS,
    TOGGLE_SMART_HOTSPOT,
];

const CONNECT_TO_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::ConnectToWifiNetwork,
    summary: Grounded::known(
        "Connect BlueOS to a wifi network so the web interface is reachable on the LAN",
        Provenance::doc(ADV, 116, "- Choose a wifi network to connect to"),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 113, "##### Wifi + Hotspot network management"),
    ),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "wifi tray scans networks and submits credentials to join the selected SSID",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::NetworkResource(NetworkResource::KnownWifiNetwork),
        Provenance::doc(
            GETTING,
            81,
            "1. First, click the wifi indicator to scan for available wif",
        ),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::doc(
                GETTING,
                81,
                "1. First, click the wifi indicator to scan for available wif",
            ),
            None,
        ),
        operator_step(
            "Scan for available wifi networks",
            Some(sourced_route(
                HttpMethod::Get,
                "/scan",
                Some("v1.0"),
                63,
                "@app.get(\"/scan\", response_model=List[ScannedWifiNetwork], s",
            )),
            Provenance::doc(
                GETTING,
                81,
                "1. First, click the wifi indicator to scan for available wif",
            ),
            Some(source_outcome(
                200,
                63,
                "@app.get(\"/scan\", response_model=List[ScannedWifiNetwork], s",
            )),
        ),
        operator_step(
            "Select the desired network from the scan results",
            None,
            Provenance::doc(
                GETTING,
                84,
                "1. Select the desired network, type in the password, and cli",
            ),
            None,
        ),
        operator_step(
            "Enter the network password when prompted and click Connect",
            Some(sourced_route(
                HttpMethod::Post,
                "/connect",
                Some("v1.0"),
                82,
                "@app.post(\"/connect\", summary=\"Connect to wifi network.\")",
            )),
            Provenance::doc(
                GETTING,
                84,
                "1. Select the desired network, type in the password, and cli",
            ),
            Some(source_outcome(
                200,
                84,
                "async def connect(credentials: WifiCredentials, hidden: bool",
            )),
        ),
    ]),
    availability: PRESENCE_CONNECT_TO_WIFI_NETWORK,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "POST /connect joins a WLAN and can change the active route to the web UI",
        ),
    ),
    chains_from: None,
};

const CONNECT_TO_HIDDEN_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::ConnectToHiddenWifiNetwork,
    summary: Grounded::known(
        "Connect BlueOS to a hidden wifi network by entering its SSID and password",
        Provenance::doc("content/usage/overview/index.md", 117, "| [**WIFI Manager**](../advanced/#indicators-and-n"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113, "##### Wifi + Hotspot network manag")),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "connection dialog posts /connect with hidden=true for an operator-entered SSID",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::NetworkResource(NetworkResource::KnownWifiNetwork),
        Provenance::doc("content/usage/overview/index.md", 117, "| [**WIFI Manager**](../advanced/#indicators-and-n"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12, "<v-card"),
            None,
        ),
        operator_step(
            "Enter the hidden network SSID and password, then click Connect",
            Some(sourced_route(HttpMethod::Post, "/connect", Some("v1.0"), 82, "@app.post(\"/connect\", summary=\"Co")),
            Provenance::source(CONNECTION_DIALOG, 211, "params: { hidden: this.is_hidden },"),
            Some(source_outcome(200, 84, "async def connect(credentials: WifiCredentials, hidden: bool")),
        ),
    ]),
    availability: PRESENCE_CONNECT_TO_HIDDEN_WIFI_NETWORK,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "POST /connect with hidden=true joins a WLAN and can change the active route to the web UI",
        ),
    ),
    chains_from: None,
};

const DISCONNECT_FROM_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::DisconnectFromWifiNetwork,
    summary: Grounded::known(
        "Disconnect BlueOS from the currently connected wifi network",
        Provenance::source(DISCONNECTION_DIALOG, 68, "@click=\"disconnectFromWifiNetwork\""),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113, "##### Wifi + Hotspot network manag")),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DisconnectWifiNetwork,
        "wifi tray disconnects the active wlan association from the current-network card",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataRequirement::WifiCurrentlyConnected),
        Provenance::source(WIFI_MANAGER, 47, "v-if=\"current_network\""),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12, "<v-card"),
            None,
        ),
        operator_step(
            "Click the connected network card at the top of the wifi list",
            None,
            Provenance::source(WIFI_MANAGER, 51, "@click=\"openDisconnectionDialog\""),
            None,
        ),
        operator_step(
            "Confirm disconnect in the disconnection dialog",
            Some(sourced_route(HttpMethod::Get, "/disconnect", Some("v1.0"), 102, "@app.get(\"/disconnect\", summary")),
            Provenance::source(DISCONNECTION_DIALOG, 135, "async disconnectFromWifiNetwork(): Promise<void> {"),
            Some(source_outcome(200, 102, "@app.get(\"/disconnect\", summary=\"Disconnect from wifi networ")),
        ),
    ]),
    availability: PRESENCE_DISCONNECT_FROM_WIFI_NETWORK,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "GET /disconnect drops the active wlan association and can block UI access until reconnect",
        ),
    ),
    chains_from: None,
};

const FORGET_SAVED_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::ForgetSavedWifiNetwork,
    summary: Grounded::known(
        "Forget a saved wifi network so BlueOS no longer auto-connects to it",
        Provenance::doc(
            ADV,
            123,
            "- Forget, connect to, or a force a new password for a saved ",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 113, "##### Wifi + Hotspot network management"),
    ),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RemoveSavedWifiNetwork,
        "connection dialog removes a stored SSID from saved networks",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataRequirement::WifiNetworkSaved),
        Provenance::source(CONNECTION_DIALOG, 76, "v-if=\"network.saved\""),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12, "<v-card"),
            None,
        ),
        operator_step(
            "Select the saved network from the wifi list",
            None,
            Provenance::doc(
                ADV,
                123,
                "- Forget, connect to, or a force a new password for a saved ",
            ),
            None,
        ),
        operator_step(
            "Click Forget in the connection dialog",
            Some(sourced_route(
                HttpMethod::Post,
                "/remove",
                Some("v1.0"),
                89,
                "@app.post(\"/remove\", summary=\"Remove saved wifi network.\")",
            )),
            Provenance::source(
                CONNECTION_DIALOG,
                226,
                "async removeSavedWifiNetwork(): Promise<void> {",
            ),
            Some(source_outcome(
                200,
                91,
                "async def remove(ssid: str) -> Any:",
            )),
        ),
    ]),
    availability: PRESENCE_FORGET_SAVED_WIFI_NETWORK,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "POST /remove deletes saved SSID credentials restorable by reconnecting",
        ),
    ),
    chains_from: None,
};

const FORCE_WIFI_NETWORK_PASSWORD: UserJourney = UserJourney {
    id: JourneyId::ForceWifiNetworkPassword,
    summary: Grounded::known(
        "Force a new password when reconnecting to a saved wifi network",
        Provenance::doc(ADV, 123, "- Forget, connect to, or a force a new password for a saved "),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113, "##### Wifi + Hotspot network manag")),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "connection dialog Force new password re-submits credentials via POST /connect",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataRequirement::WifiNetworkSaved),
        Provenance::source(CONNECTION_DIALOG, 76, "v-if=\"network.saved\""),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12, "<v-card"),
            None,
        ),
        operator_step(
            "Select the saved network and click Force new password",
            None,
            Provenance::source(CONNECTION_DIALOG, 62, "Force new password"),
            None,
        ),
        operator_step(
            "Enter the new password and click Connect",
            Some(sourced_route(HttpMethod::Post, "/connect", Some("v1.0"), 82, "@app.post(\"/connect\", summary=\"Co")),
            Provenance::source(CONNECTION_DIALOG, 199, "this.force_password = !this.force_password"),
            Some(source_outcome(200, 84, "async def connect(credentials: WifiCredentials, hidden: bool")),
        ),
    ]),
    availability: PRESENCE_FORCE_WIFI_NETWORK_PASSWORD,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "POST /connect with Force updates the stored password restorable by forcing the prior value",
        ),
    ),
    chains_from: None,
};

const RECONNECT_TO_SAVED_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::ReconnectToSavedWifiNetwork,
    summary: Grounded::known(
        "Reconnect to a saved wifi network using the stored password",
        Provenance::doc(
            ADV,
            123,
            "- Forget, connect to, or a force a new password for a saved ",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 113, "##### Wifi + Hotspot network management"),
    ),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "connection dialog connects to a saved SSID without re-entering the password",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataRequirement::WifiNetworkSaved),
        Provenance::source(CONNECTION_DIALOG, 76, "v-if=\"network.saved\""),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12, "<v-card"),
            None,
        ),
        operator_step(
            "Select the saved network and click Connect without entering a password",
            Some(sourced_route(
                HttpMethod::Post,
                "/connect",
                Some("v1.0"),
                82,
                "@app.post(\"/connect\", summary=\"Connect to wifi network.\")",
            )),
            Provenance::doc(
                ADV,
                123,
                "- Forget, connect to, or a force a new password for a saved ",
            ),
            Some(source_outcome(
                200,
                84,
                "async def connect(credentials: WifiCredentials, hidden: bool",
            )),
        ),
    ]),
    availability: PRESENCE_RECONNECT_TO_SAVED_WIFI_NETWORK,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "POST /connect re-establishes wlan association using saved credentials",
        ),
    ),
    chains_from: None,
};

const REJECT_INVALID_WIFI_CREDENTIALS: UserJourney = UserJourney {
    id: JourneyId::RejectInvalidWifiCredentials,
    summary: Grounded::known(
        "Attempt to join a wifi network with the wrong password and see the connection fail",
        Provenance::source(
            CONNECTION_DIALOG,
            222,
            "message = message.concat('\\n', 'Please check if the password",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 113, "##### Wifi + Hotspot network management"),
    ),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "failed POST /connect surfaces a check-password error to the operator",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::NetworkResource(NetworkResource::KnownWifiNetwork),
        Provenance::source(
            CONNECTION_DIALOG,
            222,
            "message = message.concat('\\n', 'Please check if the password",
        ),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12, "<v-card"),
            None,
        ),
        operator_step(
            "Select a network, enter an incorrect password, and click Connect",
            Some(sourced_route(
                HttpMethod::Post,
                "/connect",
                Some("v1.0"),
                82,
                "@app.post(\"/connect\", summary=\"Connect to wifi network.\")",
            )),
            Provenance::source(
                CONNECTION_DIALOG,
                222,
                "message = message.concat('\\n', 'Please check if the password",
            ),
            Some(source_outcome(
                500,
                84,
                "async def connect(credentials: WifiCredentials, hidden: bool",
            )),
        ),
    ]),
    availability: PRESENCE_REJECT_INVALID_WIFI_CREDENTIALS,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "POST /connect with a wrong password rejects without persisting a saved association",
        ),
    ),
    chains_from: None,
};

const DETECT_WIFI_AP_LOSS: UserJourney = UserJourney {
    id: JourneyId::DetectWifiApLoss,
    summary: Grounded::known(
        "Notice when the associated wifi access point disappears while BlueOS is connected",
        Provenance::source(WIFI_MANAGER, 47, "v-if=\"current_network\""),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 113, "##### Wifi + Hotspot network management"),
    ),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::GetWifiStatus,
        "wifi updater poll of GET /status reflects loss of the associated SSID",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Data(DataRequirement::WifiCurrentlyConnected),
            Provenance::source(WIFI_MANAGER, 47, "v-if=\"current_network\""),
        ),
        GroundedItem::new(
            Precondition::NetworkResource(NetworkResource::KnownWifiNetwork),
            Provenance::source(WIFI_MANAGER, 47, "v-if=\"current_network\""),
        ),
    ]),
    steps: GroundedSet::known(&[operator_step(
        "While connected, observe the wifi tray when the access point goes away",
        Some(sourced_route(
            HttpMethod::Get,
            "/status",
            Some("v1.0"),
            53,
            "@app.get(\"/status\", summary=\"Retrieve status of wifi manager",
        )),
        Provenance::source(
            "core/frontend/src/components/wifi/WifiUpdater.vue",
            1,
            "<template>",
        ),
        Some(source_outcome(
            200,
            53,
            "@app.get(\"/status\", summary=\"Retrieve status of wifi manager",
        )),
    )]),
    availability: PRESENCE_DETECT_WIFI_AP_LOSS,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "Operator polls GET /status to observe AP loss without issuing a mutating wifi request",
        ),
    ),
    chains_from: Some(JourneyId::ConnectToWifiNetwork),
};

const AUTOCONNECT_TO_SAVED_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::AutoconnectToSavedWifiNetwork,
    summary: Grounded::known(
        "Automatically reconnect to a saved wifi network when its access point returns",
        Provenance::doc(
            ADV,
            123,
            "- Forget, connect to, or a force a new password for a saved ",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 113, "##### Wifi + Hotspot network management"),
    ),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "wpa_supplicant autoconnect rejoins the saved SSID without a new POST /connect",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Data(DataRequirement::WifiNetworkSaved),
            Provenance::source(CONNECTION_DIALOG, 76, "v-if=\"network.saved\""),
        ),
        GroundedItem::new(
            Precondition::NetworkResource(NetworkResource::KnownWifiNetwork),
            Provenance::doc(
                ADV,
                123,
                "- Forget, connect to, or a force a new password for a saved ",
            ),
        ),
    ]),
    steps: GroundedSet::known(&[operator_step(
        "After the known access point returns, wait for BlueOS to reassociate",
        Some(sourced_route(
            HttpMethod::Get,
            "/status",
            Some("v1.0"),
            53,
            "@app.get(\"/status\", summary=\"Retrieve status of wifi manager",
        )),
        Provenance::doc(
            ADV,
            123,
            "- Forget, connect to, or a force a new password for a saved ",
        ),
        Some(source_outcome(
            200,
            53,
            "@app.get(\"/status\", summary=\"Retrieve status of wifi manager",
        )),
    )]),
    availability: PRESENCE_AUTOCONNECT_TO_SAVED_WIFI_NETWORK,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "wpa_supplicant auto-reassociates when the saved AP returns, changing wlan state",
        ),
    ),
    chains_from: Some(JourneyId::DetectWifiApLoss),
};

const TOGGLE_HOTSPOT: UserJourney = UserJourney {
    id: JourneyId::ToggleHotspot,
    summary: Grounded::known(
        "Turn the BlueOS wireless hotspot on or off from the wifi tray",
        Provenance::doc(
            ADV,
            127,
            "- Configure or turn on/off the BlueOS wireless hotspot, or d",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 113, "##### Wifi + Hotspot network management"),
    ),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ToggleHotspot,
        "wifi tray hotspot button enables or disables the onboard access point",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::NetworkResource(NetworkResource::HotspotCapable),
        Provenance::doc(
            ADV,
            127,
            "- Configure or turn on/off the BlueOS wireless hotspot, or d",
        ),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12, "<v-card"),
            None,
        ),
        operator_step(
            "Click the hotspot toggle button in the wifi tray toolbar",
            Some(sourced_route(
                HttpMethod::Post,
                "/hotspot",
                Some("v1.0"),
                126,
                "@app.post(\"/hotspot\", summary=\"Enable/disable hotspot.\")",
            )),
            Provenance::source(WIFI_MANAGER, 262, "async toggleHotspot(): Promise<void> {"),
            Some(source_outcome(
                200,
                128,
                "async def toggle_hotspot(enable: bool) -> Any:",
            )),
        ),
    ]),
    availability: PRESENCE_TOGGLE_HOTSPOT,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "POST /hotspot toggles onboard AP mode and changes how clients reach BlueOS",
        ),
    ),
    chains_from: None,
};

const CONFIGURE_HOTSPOT_CREDENTIALS: UserJourney = UserJourney {
    id: JourneyId::ConfigureHotspotCredentials,
    summary: Grounded::known(
        "Set the BlueOS hotspot SSID and password shown to connecting devices",
        Provenance::doc(ADV, 127, "- Configure or turn on/off the BlueOS wireless hotspot, or d"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113, "##### Wifi + Hotspot network manag")),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::SetHotspotCredentials,
        "wifi settings dialog persists hotspot SSID and password",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::NetworkResource(NetworkResource::HotspotCapable),
        Provenance::doc(ADV, 127, "- Configure or turn on/off the BlueOS wireless hotspot, or d"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12, "<v-card"),
            None,
        ),
        operator_step(
            "Open wifi settings from the cog button in the tray toolbar",
            None,
            Provenance::source(WIFI_MANAGER, 35, "v-tooltip=\"'Settings'\""),
            None,
        ),
        operator_step(
            "Edit the hotspot SSID and password, then click Save",
            Some(sourced_route(HttpMethod::Post, "/hotspot_credentials", Some("v1.0"), 152, "@app.post(\"/hotspot_c")),
            Provenance::source(WIFI_SETTINGS, 104, "const credentials: NetworkCredentials = { ssid: this.inputed"),
            Some(source_outcome(200, 154, "async def set_hotspot_credentials(credentials: WifiCredentia")),
        ),
    ]),
    availability: PRESENCE_CONFIGURE_HOTSPOT_CREDENTIALS,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "POST /hotspot_credentials changes hotspot SSID and password restorable by saving prior values",
        ),
    ),
    chains_from: None,
};

const TOGGLE_SMART_HOTSPOT: UserJourney = UserJourney {
    id: JourneyId::ToggleSmartHotspot,
    summary: Grounded::known(
        "Enable or disable smart-hotspot so BlueOS auto-starts its hotspot when not on wifi",
        Provenance::doc(GETTING, 33, "- By default if BlueOS does not have a wifi connection confi"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113, "##### Wifi + Hotspot network manag")),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ToggleSmartHotspot,
        "wifi settings dialog enables auto-hotspot when no known network is connected",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::NetworkResource(NetworkResource::HotspotCapable),
        Provenance::doc(GETTING, 33, "- By default if BlueOS does not have a wifi connection confi"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12, "<v-card"),
            None,
        ),
        operator_step(
            "Open wifi settings from the cog button in the tray toolbar",
            None,
            Provenance::source(WIFI_MANAGER, 35, "v-tooltip=\"'Settings'\""),
            None,
        ),
        operator_step(
            "Toggle Enable smart-hotspot and click Save",
            Some(sourced_route(HttpMethod::Post, "/smart_hotspot", Some("v1.0"), 135, "@app.post(\"/smart_hotspot\",")),
            Provenance::source(WIFI_SETTINGS, 117, "await back_axios({"),
            Some(source_outcome(200, 137, "def toggle_smart_hotspot(enable: bool) -> Any:")),
        ),
    ]),
    availability: PRESENCE_TOGGLE_SMART_HOTSPOT,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "POST /smart_hotspot toggles auto-hotspot preference restorable by saving the prior setting",
        ),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const WIFI_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Wifi,
    Provenance::doc(
        ADV,
        114,
        "{{ service(service=\"Wifi Manager\", port=9000, link=\"/service",
    ),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Wifi,
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
        Provenance::source(WIFI_MAIN, line, anchor),
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
        Provenance::source(WIFI_MAIN, line, anchor),
    )
}
