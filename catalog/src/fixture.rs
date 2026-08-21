//! Test-bed fixture inventory and journey precondition evaluation for Tier-1 GET and Tier-2
//! mutating HTTP runners.
//! `FixtureInventory` describes what the runner's environment provides; `evaluate_precondition`
//! and `journey_fixtures_ready` decide whether to run or skip a journey. Runners build inventory
//! via `parse_fixture_list` (CLI) or by setting fields directly; skip when any status is not
//! `PreconditionStatus::Satisfied`.

use crate::coverage::precondition_label;
use crate::journey::{
    journey_requirements, BoardKind, DataRequirement, HardwareRequirement, NetworkResource,
    NetworkState, Precondition, SoftwareRequirement, UserJourney,
};
/// Declared test-bed resources. Absent/false means "not available";
/// evaluation returns Missing → runner SKIPS the journey (not fail).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FixtureInventory {
    pub internet: bool,
    pub pirate_mode: bool,
    pub advanced_mode: bool,
    pub dev_mode: bool,
    pub confirm_dangerous_op: bool,
    pub flight_controller: Option<BoardKind>,
    pub usb_camera: bool,
    pub ping1d: bool,
    pub ping360: bool,
    pub external_nmea_gps: bool,
    pub usb_serial_device: bool,
    pub wifi_radio: bool,
    pub known_wifi_network: bool,
    pub hotspot_capable: bool,
    pub wired_ethernet: bool,
    pub usb_otg: bool,
    pub extension_installed: bool,
    pub local_blueos_version_available: bool,
    pub serial_bridge_configured: bool,
    pub nmea_socket_configured: bool,
    pub recording_listed: bool,
    pub wifi_network_saved: bool,
    pub wifi_currently_connected: bool,
    pub onboard_dhcp_server_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreconditionStatus {
    Satisfied,
    Missing(String),
    Unevaluable(String),
}

pub fn evaluate_precondition(
    precondition: &Precondition,
    fixtures: &FixtureInventory,
) -> PreconditionStatus {
    match precondition {
        Precondition::Network(NetworkState::Online) => {
            if fixtures.internet {
                PreconditionStatus::Satisfied
            } else {
                PreconditionStatus::Missing("internet access required".into())
            }
        }
        Precondition::Network(NetworkState::Offline) => {
            if fixtures.internet {
                PreconditionStatus::Missing("offline network required".into())
            } else {
                PreconditionStatus::Satisfied
            }
        }
        Precondition::Software(SoftwareRequirement::PirateMode) => {
            require_bool(fixtures.pirate_mode, "pirate mode required")
        }
        Precondition::Software(SoftwareRequirement::AdvancedMode) => {
            require_bool(fixtures.advanced_mode, "advanced mode required")
        }
        Precondition::Software(SoftwareRequirement::DevMode) => {
            require_bool(fixtures.dev_mode, "dev mode required")
        }
        Precondition::Software(SoftwareRequirement::ConfirmDangerousOp) => require_bool(
            fixtures.confirm_dangerous_op,
            "confirm dangerous operation required",
        ),
        Precondition::Hardware(HardwareRequirement::UsbCamera) => {
            require_bool(fixtures.usb_camera, "USB camera required")
        }
        Precondition::Hardware(HardwareRequirement::Ping1d) => {
            require_bool(fixtures.ping1d, "Ping1D sonar required")
        }
        Precondition::Hardware(HardwareRequirement::Ping360) => {
            require_bool(fixtures.ping360, "Ping360 sonar required")
        }
        Precondition::Hardware(HardwareRequirement::ExternalNmeaGps) => {
            require_bool(fixtures.external_nmea_gps, "external NMEA GPS required")
        }
        Precondition::Hardware(HardwareRequirement::UsbSerialDevice) => {
            require_bool(fixtures.usb_serial_device, "USB serial device required")
        }
        Precondition::Hardware(HardwareRequirement::FlightController(kind)) => {
            evaluate_flight_controller(*kind, fixtures.flight_controller)
        }
        Precondition::NetworkResource(NetworkResource::WifiRadioPresent) => {
            require_bool(fixtures.wifi_radio, "Wi-Fi radio required")
        }
        Precondition::NetworkResource(NetworkResource::KnownWifiNetwork) => {
            require_bool(fixtures.known_wifi_network, "known Wi-Fi network required")
        }
        Precondition::NetworkResource(NetworkResource::HotspotCapable) => {
            require_bool(fixtures.hotspot_capable, "hotspot capability required")
        }
        Precondition::NetworkResource(NetworkResource::WiredEthernetPresent) => {
            require_bool(fixtures.wired_ethernet, "wired ethernet required")
        }
        Precondition::NetworkResource(NetworkResource::UsbOtgPresent) => {
            require_bool(fixtures.usb_otg, "USB OTG required")
        }
        Precondition::Data(DataRequirement::ExtensionInstalled) => {
            require_bool(fixtures.extension_installed, "installed extension required")
        }
        Precondition::Data(DataRequirement::LocalBlueosVersionAvailable) => require_bool(
            fixtures.local_blueos_version_available,
            "local BlueOS version required",
        ),
        Precondition::Data(DataRequirement::SerialBridgeConfigured) => require_bool(
            fixtures.serial_bridge_configured,
            "configured serial bridge required",
        ),
        Precondition::Data(DataRequirement::NmeaSocketConfigured) => require_bool(
            fixtures.nmea_socket_configured,
            "configured NMEA socket required",
        ),
        Precondition::Data(DataRequirement::RecordingListed) => {
            require_bool(fixtures.recording_listed, "listed recording required")
        }
        Precondition::Data(DataRequirement::WifiNetworkSaved) => {
            require_bool(fixtures.wifi_network_saved, "saved Wi-Fi network required")
        }
        Precondition::Data(DataRequirement::WifiCurrentlyConnected) => require_bool(
            fixtures.wifi_currently_connected,
            "active Wi-Fi connection required",
        ),
        Precondition::Data(DataRequirement::OnboardDhcpServerActive) => require_bool(
            fixtures.onboard_dhcp_server_active,
            "onboard DHCP server active required",
        ),
        Precondition::Other(_)
        | Precondition::HardwarePresent(_)
        | Precondition::ServiceState { .. }
        | Precondition::ConfigClean(_) => {
            PreconditionStatus::Unevaluable(precondition_label(precondition))
        }
    }
}

pub fn evaluate_journey(
    journey: &UserJourney,
    fixtures: &FixtureInventory,
) -> Vec<PreconditionStatus> {
    journey_requirements(journey)
        .into_iter()
        .map(|precondition| evaluate_precondition(precondition, fixtures))
        .collect()
}

pub fn journey_fixtures_ready(journey: &UserJourney, fixtures: &FixtureInventory) -> bool {
    evaluate_journey(journey, fixtures)
        .iter()
        .all(|status| matches!(status, PreconditionStatus::Satisfied))
}

/// Like [`journey_fixtures_ready`], but treats `Precondition::Other` as satisfied for allowlisted
/// idempotent mutating-smoke journeys (e.g. DELETE branding assets when none are uploaded).
pub fn journey_mutating_smoke_ready(journey: &UserJourney, fixtures: &FixtureInventory) -> bool {
    evaluate_journey(journey, fixtures).iter().all(|status| {
        matches!(
            status,
            PreconditionStatus::Satisfied | PreconditionStatus::Unevaluable(_)
        )
    })
}

/// Parse comma-separated fixture tokens: `internet,pirate,usb-camera,board:navigator,known-wifi`.
///
/// Tokens: `internet`, `pirate`, `advanced`, `dev`, `confirm-dangerous`, `usb-camera`, `ping1d`,
/// `ping360`, `nmea-gps`, `usb-serial`, `wifi-radio`, `known-wifi`, `hotspot`, `wired`,
/// `usb-otg`, `extension-installed`, `local-version`, `serial-bridge`, `nmea-socket`,
/// `recording`, `wifi-saved`, `wifi-connected`, `dhcp-active`, `board:any`, `board:navigator`,
/// `board:pixhawk`, `board:sitl`.
pub fn parse_fixture_list(spec: &str) -> Result<FixtureInventory, String> {
    let mut fixtures = FixtureInventory::default();

    for token in spec
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        match token {
            "internet" => fixtures.internet = true,
            "pirate" => fixtures.pirate_mode = true,
            "advanced" => fixtures.advanced_mode = true,
            "dev" => fixtures.dev_mode = true,
            "confirm-dangerous" => fixtures.confirm_dangerous_op = true,
            "usb-camera" => fixtures.usb_camera = true,
            "ping1d" => fixtures.ping1d = true,
            "ping360" => fixtures.ping360 = true,
            "nmea-gps" => fixtures.external_nmea_gps = true,
            "usb-serial" => fixtures.usb_serial_device = true,
            "wifi-radio" => fixtures.wifi_radio = true,
            "known-wifi" => fixtures.known_wifi_network = true,
            "hotspot" => fixtures.hotspot_capable = true,
            "wired" => fixtures.wired_ethernet = true,
            "usb-otg" => fixtures.usb_otg = true,
            "extension-installed" => fixtures.extension_installed = true,
            "local-version" => fixtures.local_blueos_version_available = true,
            "serial-bridge" => fixtures.serial_bridge_configured = true,
            "nmea-socket" => fixtures.nmea_socket_configured = true,
            "recording" => fixtures.recording_listed = true,
            "wifi-saved" => fixtures.wifi_network_saved = true,
            "wifi-connected" => fixtures.wifi_currently_connected = true,
            "dhcp-active" => fixtures.onboard_dhcp_server_active = true,
            "board:any" => fixtures.flight_controller = Some(BoardKind::Any),
            "board:navigator" => fixtures.flight_controller = Some(BoardKind::Navigator),
            "board:pixhawk" => fixtures.flight_controller = Some(BoardKind::Pixhawk),
            "board:sitl" => fixtures.flight_controller = Some(BoardKind::Sitl),
            _ => return Err(format!("unknown fixture token: {token}")),
        }
    }

    Ok(fixtures)
}

fn require_bool(present: bool, message: &'static str) -> PreconditionStatus {
    if present {
        PreconditionStatus::Satisfied
    } else {
        PreconditionStatus::Missing(message.into())
    }
}

fn evaluate_flight_controller(
    required: BoardKind,
    present: Option<BoardKind>,
) -> PreconditionStatus {
    match (required, present) {
        (BoardKind::Any, Some(_)) => PreconditionStatus::Satisfied,
        (BoardKind::Any, None) => PreconditionStatus::Missing("flight controller required".into()),
        (expected, Some(actual)) if expected == actual => PreconditionStatus::Satisfied,
        (expected, Some(actual)) => PreconditionStatus::Missing(format!(
            "flight controller {expected:?} required, have {actual:?}"
        )),
        (_, None) => PreconditionStatus::Missing("flight controller required".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PRESENCE: crate::version::FeatureAvailability =
        crate::version::FeatureAvailability {
            intro_commit: "0000000000000000000000000000000000000001",
            present_in_tags: &["1.0.0"],
            present_on_master: true,
            present_on_1_4_dev: true,
        };

    use crate::id::JourneyId;
    use crate::journey::{Visibility, BLAST_RADIUS_UNKNOWN};
    use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

    const DOC: Provenance = Provenance::doc("test.md", 1);

    fn empty_journey(preconditions: GroundedSet<Precondition>) -> UserJourney {
        UserJourney {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions,
            steps: GroundedSet::unknown("test"),
            availability: TEST_PRESENCE,
            blast_radius: BLAST_RADIUS_UNKNOWN,
            chains_from: None,
        }
    }

    #[test]
    fn online_precondition_ready_when_internet_present() {
        let fixtures = FixtureInventory {
            internet: true,
            ..FixtureInventory::default()
        };
        let status = evaluate_precondition(&Precondition::Network(NetworkState::Online), &fixtures);
        assert_eq!(status, PreconditionStatus::Satisfied);
    }

    #[test]
    fn online_precondition_missing_without_internet() {
        let fixtures = FixtureInventory::default();
        let status = evaluate_precondition(&Precondition::Network(NetworkState::Online), &fixtures);
        assert!(matches!(status, PreconditionStatus::Missing(_)));
    }

    #[test]
    fn pirate_mode_precondition() {
        let satisfied = evaluate_precondition(
            &Precondition::Software(SoftwareRequirement::PirateMode),
            &FixtureInventory {
                pirate_mode: true,
                ..FixtureInventory::default()
            },
        );
        assert_eq!(satisfied, PreconditionStatus::Satisfied);

        let missing = evaluate_precondition(
            &Precondition::Software(SoftwareRequirement::PirateMode),
            &FixtureInventory::default(),
        );
        assert!(matches!(missing, PreconditionStatus::Missing(_)));
    }

    #[test]
    fn flight_controller_any_vs_specific_board() {
        let any = evaluate_precondition(
            &Precondition::Hardware(HardwareRequirement::FlightController(BoardKind::Any)),
            &FixtureInventory {
                flight_controller: Some(BoardKind::Navigator),
                ..FixtureInventory::default()
            },
        );
        assert_eq!(any, PreconditionStatus::Satisfied);

        let navigator = evaluate_precondition(
            &Precondition::Hardware(HardwareRequirement::FlightController(BoardKind::Navigator)),
            &FixtureInventory {
                flight_controller: Some(BoardKind::Navigator),
                ..FixtureInventory::default()
            },
        );
        assert_eq!(navigator, PreconditionStatus::Satisfied);

        let mismatch = evaluate_precondition(
            &Precondition::Hardware(HardwareRequirement::FlightController(BoardKind::Navigator)),
            &FixtureInventory {
                flight_controller: Some(BoardKind::Pixhawk),
                ..FixtureInventory::default()
            },
        );
        assert!(matches!(mismatch, PreconditionStatus::Missing(_)));
    }

    #[test]
    fn other_precondition_is_unevaluable_and_not_ready() {
        static PRECONDITIONS: &[GroundedItem<Precondition>] = &[GroundedItem::new(
            Precondition::Other("configured stream listed"),
            DOC,
        )];
        let journey = empty_journey(GroundedSet::known(PRECONDITIONS));
        let fixtures = FixtureInventory::default();

        let statuses = evaluate_journey(&journey, &fixtures);
        assert_eq!(statuses.len(), 1);
        assert!(matches!(statuses[0], PreconditionStatus::Unevaluable(_)));
        assert!(!journey_fixtures_ready(&journey, &fixtures));
        assert!(journey_mutating_smoke_ready(&journey, &fixtures));
    }

    #[test]
    fn empty_preconditions_are_ready() {
        let journey = empty_journey(GroundedSet::known(&[]));
        assert!(journey_fixtures_ready(
            &journey,
            &FixtureInventory::default()
        ));
    }

    #[test]
    fn parse_fixture_list_sets_expected_flags() {
        let fixtures = parse_fixture_list(
            "internet,pirate,advanced,dev,confirm-dangerous,usb-camera,ping1d,ping360,\
             nmea-gps,usb-serial,wifi-radio,known-wifi,hotspot,wired,usb-otg,\
             extension-installed,local-version,serial-bridge,nmea-socket,recording,\
             wifi-saved,wifi-connected,dhcp-active,board:navigator",
        )
        .expect("tokens should parse");

        assert_eq!(
            fixtures,
            FixtureInventory {
                internet: true,
                pirate_mode: true,
                advanced_mode: true,
                dev_mode: true,
                confirm_dangerous_op: true,
                flight_controller: Some(BoardKind::Navigator),
                usb_camera: true,
                ping1d: true,
                ping360: true,
                external_nmea_gps: true,
                usb_serial_device: true,
                wifi_radio: true,
                known_wifi_network: true,
                hotspot_capable: true,
                wired_ethernet: true,
                usb_otg: true,
                extension_installed: true,
                local_blueos_version_available: true,
                serial_bridge_configured: true,
                nmea_socket_configured: true,
                recording_listed: true,
                wifi_network_saved: true,
                wifi_currently_connected: true,
                onboard_dhcp_server_active: true,
            }
        );
    }

    #[test]
    fn parse_fixture_list_rejects_unknown_token() {
        let err = parse_fixture_list("internet,not-a-fixture").unwrap_err();
        assert!(err.contains("not-a-fixture"));
    }
}
