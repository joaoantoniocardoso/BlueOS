use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
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

#[allow(dead_code)]
pub fn journeys() -> Vec<UserJourney> {
    vec![
        connect_to_wifi_network(),
        disconnect_from_wifi_network(),
        forget_saved_wifi_network(),
        toggle_hotspot(),
        configure_hotspot_credentials(),
        toggle_smart_hotspot(),
    ]
}

fn connect_to_wifi_network() -> UserJourney {
    UserJourney {
        id: JourneyId("connect_to_wifi_network".into()),
        summary: Grounded::known(
            "Connect BlueOS to a wifi network so the web interface is reachable on the LAN".into(),
            Provenance::doc(ADV, 116),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
        services: wifi_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "connect_wifi_network",
            "wifi tray scans networks and submits credentials to join the selected SSID",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
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
                None,
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
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn disconnect_from_wifi_network() -> UserJourney {
    UserJourney {
        id: JourneyId("disconnect_from_wifi_network".into()),
        summary: Grounded::known(
            "Disconnect BlueOS from the currently connected wifi network".into(),
            Provenance::source(DISCONNECTION_DIALOG, 68),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
        services: wifi_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "disconnect_wifi_network",
            "wifi tray disconnects the active wlan association from the current-network card",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("BlueOS is connected to a wifi network".into()),
            Provenance::source(WIFI_MANAGER, 47),
        )]),
        steps: GroundedSet::known(vec![
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
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn forget_saved_wifi_network() -> UserJourney {
    UserJourney {
        id: JourneyId("forget_saved_wifi_network".into()),
        summary: Grounded::known(
            "Forget a saved wifi network so BlueOS no longer auto-connects to it".into(),
            Provenance::doc(ADV, 123),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
        services: wifi_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "remove_saved_wifi_network",
            "connection dialog removes a stored SSID from saved networks",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("The target network is already saved on the vehicle".into()),
            Provenance::source(CONNECTION_DIALOG, 76),
        )]),
        steps: GroundedSet::known(vec![
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
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn toggle_hotspot() -> UserJourney {
    UserJourney {
        id: JourneyId("toggle_hotspot".into()),
        summary: Grounded::known(
            "Turn the BlueOS wireless hotspot on or off from the wifi tray".into(),
            Provenance::doc(ADV, 127),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
        services: wifi_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "toggle_hotspot",
            "wifi tray hotspot button enables or disables the onboard access point",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
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
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn configure_hotspot_credentials() -> UserJourney {
    UserJourney {
        id: JourneyId("configure_hotspot_credentials".into()),
        summary: Grounded::known(
            "Set the BlueOS hotspot SSID and password shown to connecting devices".into(),
            Provenance::doc(ADV, 127),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
        services: wifi_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "set_hotspot_credentials",
            "wifi settings dialog persists hotspot SSID and password",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
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
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn toggle_smart_hotspot() -> UserJourney {
    UserJourney {
        id: JourneyId("toggle_smart_hotspot".into()),
        summary: Grounded::known(
            "Enable or disable smart-hotspot so BlueOS auto-starts its hotspot when not on wifi"
                .into(),
            Provenance::doc(GETTING, 33),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 113)),
        services: wifi_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "toggle_smart_hotspot",
            "wifi settings dialog enables auto-hotspot when no known network is connected",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
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
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn wifi_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId("wifi".into()),
        Provenance::doc(ADV, 114),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId("wifi".into()),
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
        Provenance::source(WIFI_MAIN, line),
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
