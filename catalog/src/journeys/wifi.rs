use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, DataRequirement, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome,
    UserJourney, Visibility,
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
        Provenance::doc(ADV, 116),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "wifi tray scans networks and submits credentials to join the selected SSID",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::doc(GETTING, 81),
            None,
        ),
        operator_step(
            "Scan for available wifi networks",
            Some(sourced_route(HttpMethod::Get, "/scan", Some("v1.0"), 63)),
            Provenance::doc(GETTING, 81),
            Some(source_outcome(200, 63)),
        ),
        operator_step(
            "Select the desired network from the scan results",
            None,
            Provenance::doc(GETTING, 84),
            None,
        ),
        operator_step(
            "Enter the network password when prompted and click Connect",
            Some(sourced_route(
                HttpMethod::Post,
                "/connect",
                Some("v1.0"),
                82,
            )),
            Provenance::doc(GETTING, 84),
            Some(source_outcome(200, 84)),
        ),
    ]),
    availability: PRESENCE_CONNECT_TO_WIFI_NETWORK,
    chains_from: None,
};

const CONNECT_TO_HIDDEN_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::ConnectToHiddenWifiNetwork,
    summary: Grounded::known(
        "Connect BlueOS to a hidden wifi network by entering its SSID and password",
        Provenance::doc("content/usage/overview/index.md", 117),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "connection dialog posts /connect with hidden=true for an operator-entered SSID",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12),
            None,
        ),
        operator_step(
            "Enter the hidden network SSID and password, then click Connect",
            Some(sourced_route(
                HttpMethod::Post,
                "/connect",
                Some("v1.0"),
                82,
            )),
            Provenance::source(CONNECTION_DIALOG, 211),
            Some(source_outcome(200, 84)),
        ),
    ]),
    availability: PRESENCE_CONNECT_TO_HIDDEN_WIFI_NETWORK,
    chains_from: None,
};

const DISCONNECT_FROM_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::DisconnectFromWifiNetwork,
    summary: Grounded::known(
        "Disconnect BlueOS from the currently connected wifi network",
        Provenance::source(DISCONNECTION_DIALOG, 68),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DisconnectWifiNetwork,
        "wifi tray disconnects the active wlan association from the current-network card",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataRequirement::WifiCurrentlyConnected),
        Provenance::source(WIFI_MANAGER, 47),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12),
            None,
        ),
        operator_step(
            "Click the connected network card at the top of the wifi list",
            None,
            Provenance::source(WIFI_MANAGER, 51),
            None,
        ),
        operator_step(
            "Confirm disconnect in the disconnection dialog",
            Some(sourced_route(
                HttpMethod::Get,
                "/disconnect",
                Some("v1.0"),
                102,
            )),
            Provenance::source(DISCONNECTION_DIALOG, 135),
            Some(source_outcome(200, 102)),
        ),
    ]),
    availability: PRESENCE_DISCONNECT_FROM_WIFI_NETWORK,
    chains_from: None,
};

const FORGET_SAVED_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::ForgetSavedWifiNetwork,
    summary: Grounded::known(
        "Forget a saved wifi network so BlueOS no longer auto-connects to it",
        Provenance::doc(ADV, 123),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RemoveSavedWifiNetwork,
        "connection dialog removes a stored SSID from saved networks",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataRequirement::WifiNetworkSaved),
        Provenance::source(CONNECTION_DIALOG, 76),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12),
            None,
        ),
        operator_step(
            "Select the saved network from the wifi list",
            None,
            Provenance::doc(ADV, 123),
            None,
        ),
        operator_step(
            "Click Forget in the connection dialog",
            Some(sourced_route(HttpMethod::Post, "/remove", Some("v1.0"), 89)),
            Provenance::source(CONNECTION_DIALOG, 226),
            Some(source_outcome(200, 91)),
        ),
    ]),
    availability: PRESENCE_FORGET_SAVED_WIFI_NETWORK,
    chains_from: None,
};

const FORCE_WIFI_NETWORK_PASSWORD: UserJourney = UserJourney {
    id: JourneyId::ForceWifiNetworkPassword,
    summary: Grounded::known(
        "Force a new password when reconnecting to a saved wifi network",
        Provenance::doc(ADV, 123),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "connection dialog Force new password re-submits credentials via POST /connect",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataRequirement::WifiNetworkSaved),
        Provenance::source(CONNECTION_DIALOG, 76),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12),
            None,
        ),
        operator_step(
            "Select the saved network and click Force new password",
            None,
            Provenance::source(CONNECTION_DIALOG, 62),
            None,
        ),
        operator_step(
            "Enter the new password and click Connect",
            Some(sourced_route(
                HttpMethod::Post,
                "/connect",
                Some("v1.0"),
                82,
            )),
            Provenance::source(CONNECTION_DIALOG, 199),
            Some(source_outcome(200, 84)),
        ),
    ]),
    availability: PRESENCE_FORCE_WIFI_NETWORK_PASSWORD,
    chains_from: None,
};

const RECONNECT_TO_SAVED_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::ReconnectToSavedWifiNetwork,
    summary: Grounded::known(
        "Reconnect to a saved wifi network using the stored password",
        Provenance::doc(ADV, 123),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "connection dialog connects to a saved SSID without re-entering the password",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataRequirement::WifiNetworkSaved),
        Provenance::source(CONNECTION_DIALOG, 76),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12),
            None,
        ),
        operator_step(
            "Select the saved network and click Connect without entering a password",
            Some(sourced_route(
                HttpMethod::Post,
                "/connect",
                Some("v1.0"),
                82,
            )),
            Provenance::doc(ADV, 123),
            Some(source_outcome(200, 84)),
        ),
    ]),
    availability: PRESENCE_RECONNECT_TO_SAVED_WIFI_NETWORK,
    chains_from: None,
};

const REJECT_INVALID_WIFI_CREDENTIALS: UserJourney = UserJourney {
    id: JourneyId::RejectInvalidWifiCredentials,
    summary: Grounded::known(
        "Attempt to join a wifi network with the wrong password and see the connection fail",
        Provenance::source(CONNECTION_DIALOG, 222),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "failed POST /connect surfaces a check-password error to the operator",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12),
            None,
        ),
        operator_step(
            "Select a network, enter an incorrect password, and click Connect",
            Some(sourced_route(
                HttpMethod::Post,
                "/connect",
                Some("v1.0"),
                82,
            )),
            Provenance::source(CONNECTION_DIALOG, 222),
            Some(source_outcome(500, 84)),
        ),
    ]),
    availability: PRESENCE_REJECT_INVALID_WIFI_CREDENTIALS,
    chains_from: None,
};

const DETECT_WIFI_AP_LOSS: UserJourney = UserJourney {
    id: JourneyId::DetectWifiApLoss,
    summary: Grounded::known(
        "Notice when the associated wifi access point disappears while BlueOS is connected",
        Provenance::source(WIFI_MANAGER, 47),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::GetWifiStatus,
        "wifi updater poll of GET /status reflects loss of the associated SSID",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataRequirement::WifiCurrentlyConnected),
        Provenance::source(WIFI_MANAGER, 47),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "While connected, observe the wifi tray when the access point goes away",
        Some(sourced_route(HttpMethod::Get, "/status", Some("v1.0"), 53)),
        Provenance::source("core/frontend/src/components/wifi/WifiUpdater.vue", 1),
        Some(source_outcome(200, 53)),
    )]),
    availability: PRESENCE_DETECT_WIFI_AP_LOSS,
    chains_from: Some(JourneyId::ConnectToWifiNetwork),
};

const AUTOCONNECT_TO_SAVED_WIFI_NETWORK: UserJourney = UserJourney {
    id: JourneyId::AutoconnectToSavedWifiNetwork,
    summary: Grounded::known(
        "Automatically reconnect to a saved wifi network when its access point returns",
        Provenance::doc(ADV, 123),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConnectWifiNetwork,
        "wpa_supplicant autoconnect rejoins the saved SSID without a new POST /connect",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataRequirement::WifiNetworkSaved),
        Provenance::source(CONNECTION_DIALOG, 76),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "After the known access point returns, wait for BlueOS to reassociate",
        Some(sourced_route(HttpMethod::Get, "/status", Some("v1.0"), 53)),
        Provenance::doc(ADV, 123),
        Some(source_outcome(200, 53)),
    )]),
    availability: PRESENCE_AUTOCONNECT_TO_SAVED_WIFI_NETWORK,
    chains_from: Some(JourneyId::DetectWifiApLoss),
};

const TOGGLE_HOTSPOT: UserJourney = UserJourney {
    id: JourneyId::ToggleHotspot,
    summary: Grounded::known(
        "Turn the BlueOS wireless hotspot on or off from the wifi tray",
        Provenance::doc(ADV, 127),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ToggleHotspot,
        "wifi tray hotspot button enables or disables the onboard access point",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12),
            None,
        ),
        operator_step(
            "Click the hotspot toggle button in the wifi tray toolbar",
            Some(sourced_route(
                HttpMethod::Post,
                "/hotspot",
                Some("v1.0"),
                126,
            )),
            Provenance::source(WIFI_MANAGER, 262),
            Some(source_outcome(200, 128)),
        ),
    ]),
    availability: PRESENCE_TOGGLE_HOTSPOT,
    chains_from: None,
};

const CONFIGURE_HOTSPOT_CREDENTIALS: UserJourney = UserJourney {
    id: JourneyId::ConfigureHotspotCredentials,
    summary: Grounded::known(
        "Set the BlueOS hotspot SSID and password shown to connecting devices",
        Provenance::doc(ADV, 127),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::SetHotspotCredentials,
        "wifi settings dialog persists hotspot SSID and password",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12),
            None,
        ),
        operator_step(
            "Open wifi settings from the cog button in the tray toolbar",
            None,
            Provenance::source(WIFI_MANAGER, 35),
            None,
        ),
        operator_step(
            "Edit the hotspot SSID and password, then click Save",
            Some(sourced_route(
                HttpMethod::Post,
                "/hotspot_credentials",
                Some("v1.0"),
                152,
            )),
            Provenance::source(WIFI_SETTINGS, 104),
            Some(source_outcome(200, 154)),
        ),
    ]),
    availability: PRESENCE_CONFIGURE_HOTSPOT_CREDENTIALS,
    chains_from: None,
};

const TOGGLE_SMART_HOTSPOT: UserJourney = UserJourney {
    id: JourneyId::ToggleSmartHotspot,
    summary: Grounded::known(
        "Enable or disable smart-hotspot so BlueOS auto-starts its hotspot when not on wifi",
        Provenance::doc(GETTING, 33),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
    services: WIFI_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ToggleSmartHotspot,
        "wifi settings dialog enables auto-hotspot when no known network is connected",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the wifi tray menu from the header bar",
            None,
            Provenance::source(WIFI_TRAY, 12),
            None,
        ),
        operator_step(
            "Open wifi settings from the cog button in the tray toolbar",
            None,
            Provenance::source(WIFI_MANAGER, 35),
            None,
        ),
        operator_step(
            "Toggle Enable smart-hotspot and click Save",
            Some(sourced_route(
                HttpMethod::Post,
                "/smart_hotspot",
                Some("v1.0"),
                135,
            )),
            Provenance::source(WIFI_SETTINGS, 117),
            Some(source_outcome(200, 137)),
        ),
    ]),
    availability: PRESENCE_TOGGLE_SMART_HOTSPOT,
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const WIFI_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Wifi,
    Provenance::doc(ADV, 114),
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
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(WIFI_MAIN, line),
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

const fn source_outcome(status: u16, line: u32) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            transition: None,
        },
        Provenance::source(WIFI_MAIN, line),
    )
}
