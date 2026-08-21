use serde::Serialize;

use crate::fixture::{evaluate_journey, FixtureInventory, PreconditionStatus};
use crate::id::JourneyId;
use crate::journey::UserJourney;
use crate::sitl_cal::{SitlRc, SITL_FRAME_CALIBRATION, SITL_FRAME_VECTORED};

pub const UI_CALIBRATION_JOURNEYS: &[JourneyId] = &[
    JourneyId::CalibrateGyroscope,
    JourneyId::CalibrateBarometer,
    JourneyId::LevelHorizon,
    JourneyId::CalibrateAccelerometer,
    JourneyId::CalibrateCompass,
    JourneyId::DetectMotorDirections,
];

pub const UI_NO_HARDWARE_JOURNEYS: &[JourneyId] = &[
    JourneyId::AccessWebTerminal,
    JourneyId::ManageBlueosFiles,
    JourneyId::InspectMavlinkMessagesInBrowser,
    JourneyId::ApplyParameterFile,
];

pub const UI_CAMERA_JOURNEYS: &[JourneyId] = &[
    JourneyId::ViewCameraStreams,
    JourneyId::ConfigureCameraStream,
    JourneyId::ConfigureVideoStream,
    JourneyId::ConfigureUvcDeviceControls,
    JourneyId::RemoveCameraStream,
];

pub const UI_EXTENSION_JOURNEYS: &[JourneyId] = &[
    JourneyId::BrowseExtensionStore,
    JourneyId::InstallExtension,
    JourneyId::InstallCustomExtension,
    JourneyId::ConfigureInstalledExtension,
    JourneyId::UninstallExtension,
    JourneyId::AddCustomManifest,
    JourneyId::EditExtensionDevVersion,
];

pub const UI_BAG_JOURNEYS: &[JourneyId] = &[JourneyId::ModifyBagDatabase];

pub const UI_VERSION_SETTINGS_JOURNEYS: &[JourneyId] = &[
    JourneyId::SwitchLocalBlueosVersion,
    JourneyId::UpdateBlueosVersion,
    JourneyId::PullBlueosVersionWithoutSwitch,
    JourneyId::DockerRegistryLogin,
    JourneyId::RenameVehicle,
];

pub const UI_NMEA_JOURNEYS: &[JourneyId] = &[
    JourneyId::ViewConfiguredNmeaSockets,
    JourneyId::AddExternalNmeaGpsSocket,
    JourneyId::RemoveConfiguredNmeaSocket,
];

pub const UI_BRIDGET_JOURNEYS: &[JourneyId] = &[JourneyId::CreateSerialToUdpBridge];

pub const UI_CABLE_GUY_JOURNEYS: &[JourneyId] = &[
    JourneyId::AcquireDynamicIpAddress,
    JourneyId::AssignStaticIpAddress,
    JourneyId::ChangeMdnsHostname,
    JourneyId::ConfigureHostDns,
    JourneyId::DisableOnboardDhcpServer,
    JourneyId::EnableOnboardDhcpServer,
    JourneyId::SetNetworkInterfacePriority,
];

pub const UI_WIFI_JOURNEYS: &[JourneyId] = &[
    JourneyId::AutoconnectToSavedWifiNetwork,
    JourneyId::ConfigureHotspotCredentials,
    JourneyId::ConnectToHiddenWifiNetwork,
    JourneyId::ConnectToWifiNetwork,
    JourneyId::DetectWifiApLoss,
    JourneyId::DisconnectFromWifiNetwork,
    JourneyId::ForceWifiNetworkPassword,
    JourneyId::ForgetSavedWifiNetwork,
    JourneyId::ReconnectToSavedWifiNetwork,
    JourneyId::RejectInvalidWifiCredentials,
    JourneyId::ToggleHotspot,
    JourneyId::ToggleSmartHotspot,
];

/// Journeys whose UI oracle is HTTP-only or non-Playwright (no `ui_plan`, not REST clones).
pub const UI_TYPED_SKIP: &[(JourneyId, &str)] = &[
    (
        JourneyId::AccessBlueosWebInterface,
        "http_is_operator_contract",
    ),
    (
        JourneyId::DiscoverBlueosOnNetwork,
        "http_is_operator_contract",
    ),
    (
        JourneyId::MonitorInternetConnectivity,
        "http_is_operator_contract",
    ),
    (
        JourneyId::ProbeInterfaceInternetConnectivity,
        "http_is_operator_contract",
    ),
    (
        JourneyId::VerifyInternetConnectivity,
        "http_is_operator_contract",
    ),
    (JourneyId::VehicleFirstBoot, "no_frontend_actor_step"),
    (JourneyId::ChangeBoard, "no_frontend_actor_step"),
    (JourneyId::RunSitlSimulation, "no_frontend_actor_step"),
    (JourneyId::StartAutopilot, "no_frontend_actor_step"),
    (JourneyId::StopAutopilot, "no_frontend_actor_step"),
    (JourneyId::RestartAutopilot, "no_frontend_actor_step"),
    (JourneyId::UpdateFirmwareOnline, "no_frontend_actor_step"),
    (JourneyId::UploadCustomFirmware, "no_frontend_actor_step"),
    (JourneyId::RestoreDefaultFirmware, "no_frontend_actor_step"),
    (JourneyId::RebootOnboardComputer, "http_only_host_reboot"),
    (JourneyId::ShutdownOnboardComputer, "hard_exclude_shutdown"),
    (JourneyId::SyncSystemTime, "no_frontend_actor_step"),
    (JourneyId::EnableLegacyCameraSupport, "would_reboot"),
    (JourneyId::ConnectPingViewerToSonar, "no_sonar"),
    (JourneyId::EnablePing1dRangefinderMavlink, "no_sonar"),
    (JourneyId::ViewDetectedSonarDevices, "no_sonar"),
    (
        JourneyId::InspectRaspberryEepromBootloader,
        "no_frontend_actor_step",
    ),
    (
        JourneyId::UpdateRaspberryEepromBootloader,
        "no_frontend_actor_step",
    ),
    (JourneyId::ResetBlueosSettings, "no_frontend_actor_step"),
    (JourneyId::RunHostCommand, "http_only_dev_shell"),
    (JourneyId::DeleteLocalBlueosVersion, "not_on_1.4-dev"),
    (JourneyId::UpdateBootstrapImage, "not_on_1.4-dev"),
    (JourneyId::RemoveSerialBridge, "usb_serial_device"),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiJourneyPlan {
    pub journey_id: String,
    pub sitl_frame: Option<&'static str>,
    pub actions: Vec<UiAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum UiAction {
    Open {
        path: &'static str,
    },
    Expect {
        text: &'static str,
    },
    Click {
        text: &'static str,
    },
    ClickIfVisible {
        text: &'static str,
    },
    WaitText {
        text: &'static str,
        timeout_ms: u64,
    },
    ExpectIframe,
    SitlRc {
        chan5: u16,
        chan6: u16,
        chan7: u16,
        chan8: u16,
    },
    Sleep {
        ms: u64,
    },
    ClickSelector {
        css: &'static str,
    },
    ClickSelectorIfVisible {
        css: &'static str,
    },
    HoverSelector {
        css: &'static str,
    },
    ExpectGone {
        text: &'static str,
        timeout_ms: u64,
    },
    PressKey {
        key: &'static str,
    },
}

pub fn ui_typed_skip_reason(id: JourneyId) -> Option<&'static str> {
    UI_TYPED_SKIP
        .iter()
        .find(|(journey_id, _)| *journey_id == id)
        .map(|(_, reason)| *reason)
}

pub fn ui_fixture_skip_reason(
    journey: &UserJourney,
    fixtures: &FixtureInventory,
) -> Option<String> {
    evaluate_journey(journey, fixtures)
        .into_iter()
        .find_map(|status| match status {
            PreconditionStatus::Missing(reason) => Some(reason),
            PreconditionStatus::Satisfied | PreconditionStatus::Unevaluable(_) => None,
        })
}

pub fn wizard_skip_plan() -> UiJourneyPlan {
    UiJourneyPlan {
        journey_id: "wizard_skip".into(),
        sitl_frame: None,
        actions: vec![UiAction::Open { path: "/" }],
    }
}

pub fn ui_plan(id: JourneyId) -> Option<UiJourneyPlan> {
    let actions = match id {
        JourneyId::CalibrateGyroscope => vec![
            open_configure("gyroscope"),
            sitl_rc(SitlRc::stop()),
            UiAction::Sleep { ms: 1500 },
            UiAction::Click {
                text: "Calibrate Gyroscopes",
            },
            UiAction::WaitText {
                text: "Calibration done.",
                timeout_ms: 20_000,
            },
        ],
        JourneyId::CalibrateBarometer => vec![
            open_configure("baro"),
            sitl_rc(SitlRc::stop()),
            UiAction::Sleep { ms: 1500 },
            UiAction::Expect {
                text: "Calibrate Barometer",
            },
            UiAction::Click { text: "Calibrate" },
            UiAction::WaitText {
                text: "Calibration done.",
                timeout_ms: 20_000,
            },
        ],
        JourneyId::LevelHorizon => vec![
            open_configure("accelerometer"),
            sitl_rc(SitlRc::attitude_deg(0, 0, 0)),
            UiAction::Sleep { ms: 2000 },
            UiAction::Click {
                text: "Level Horizon",
            },
            UiAction::Sleep { ms: 500 },
            UiAction::Click { text: "Calibrate" },
            UiAction::WaitText {
                text: "Calibration finished",
                timeout_ms: 20_000,
            },
        ],
        JourneyId::CalibrateAccelerometer => accel_plan(),
        JourneyId::CalibrateCompass => vec![
            open_configure("compass"),
            sitl_rc(SitlRc::mag_dance()),
            UiAction::Click {
                text: "Full (Onboard) Calibration",
            },
            UiAction::Click {
                text: "Start Full Calibration",
            },
            UiAction::Sleep { ms: 2000 },
            UiAction::ClickIfVisible {
                text: "Use GeoIP coordinates",
            },
            UiAction::Click { text: "Calibrate" },
            UiAction::WaitText {
                text: "Dismiss",
                timeout_ms: 180_000,
            },
        ],
        JourneyId::DetectMotorDirections => vec![
            UiAction::Open {
                path: "/vehicle/setup/pwm_outputs",
            },
            UiAction::Click {
                text: "Detect Reversed Motors",
            },
            UiAction::Click {
                text: "Start Detection",
            },
            UiAction::WaitText {
                text: "Motor direction detection is complete",
                timeout_ms: 120_000,
            },
        ],
        JourneyId::AccessWebTerminal => vec![
            UiAction::Open {
                path: "/tools/web-terminal",
            },
            UiAction::ExpectIframe,
        ],
        JourneyId::ManageBlueosFiles => vec![
            UiAction::Open {
                path: "/tools/file-browser",
            },
            UiAction::Expect { text: "My files" },
        ],
        JourneyId::InspectMavlinkMessagesInBrowser => vec![
            UiAction::Open {
                path: "/tools/mavlink-inspector",
            },
            UiAction::Expect { text: "HEARTBEAT" },
        ],
        JourneyId::ApplyParameterFile => vec![
            UiAction::Open {
                path: "/vehicle/parameters",
            },
            UiAction::WaitText {
                text: "Load",
                timeout_ms: 60_000,
            },
            UiAction::Click { text: "Load" },
            UiAction::WaitText {
                text: "Load parameter file",
                timeout_ms: 10_000,
            },
        ],
        JourneyId::ViewCameraStreams => vec![
            UiAction::Open {
                path: "/vehicle/video-manager",
            },
            UiAction::WaitText {
                text: "H264 USB Camera",
                timeout_ms: 60_000,
            },
            UiAction::Expect {
                text: "UDP Stream 0",
            },
            UiAction::Expect { text: "Add stream" },
        ],
        JourneyId::ConfigureCameraStream => {
            let mut actions = vec![
                UiAction::Open {
                    path: "/vehicle/video-manager",
                },
                UiAction::WaitText {
                    text: "Add stream",
                    timeout_ms: 60_000,
                },
            ];
            actions.extend(stream_creation_dialog_plan());
            actions
        }
        JourneyId::ConfigureVideoStream => vec![
            UiAction::Open {
                path: "/vehicle/video-manager",
            },
            UiAction::WaitText {
                text: "H264 USB Camera",
                timeout_ms: 60_000,
            },
            UiAction::ClickSelector {
                css: ".v-btn--example",
            },
            UiAction::WaitText {
                text: "Video Manager Settings",
                timeout_ms: 10_000,
            },
            UiAction::Click {
                text: "Reset Settings",
            },
            UiAction::Sleep { ms: 2000 },
        ],
        JourneyId::RemoveCameraStream => vec![
            UiAction::Open {
                path: "/vehicle/video-manager",
            },
            UiAction::WaitText {
                text: "UDP Stream 0",
                timeout_ms: 60_000,
            },
            UiAction::ClickSelector {
                css: ".stream-remove-btn",
            },
            UiAction::ExpectGone {
                text: "UDP Stream 0",
                timeout_ms: 30_000,
            },
        ],
        JourneyId::ConfigureUvcDeviceControls => vec![
            UiAction::Open {
                path: "/vehicle/video-manager",
            },
            UiAction::WaitText {
                text: "Device Controls",
                timeout_ms: 60_000,
            },
            UiAction::Click {
                text: "Device Controls",
            },
            UiAction::WaitText {
                text: "Device Controls",
                timeout_ms: 10_000,
            },
        ],
        JourneyId::BrowseExtensionStore => {
            let mut actions = open_extension_manager();
            actions.push(UiAction::ClickIfVisible { text: "Store" });
            actions.push(UiAction::ClickIfVisible { text: "Back Alley" });
            actions.push(UiAction::WaitText {
                text: "Cockpit",
                timeout_ms: 60_000,
            });
            actions
        }
        JourneyId::InstallExtension => {
            let mut actions = open_extension_manager_store_tab();
            actions.push(UiAction::ClickSelector {
                css: ".store-extension-card",
            });
            actions.push(UiAction::WaitText {
                text: "Install",
                timeout_ms: 10_000,
            });
            actions
        }
        JourneyId::InstallCustomExtension => {
            let mut actions = open_extension_manager_installed_tab();
            actions.push(UiAction::ClickSelector {
                css: ".v-main button.v-btn--fab.v-size--large",
            });
            actions.push(UiAction::ClickSelectorIfVisible {
                css: ".v-speed-dial--is-active .mdi-code-braces",
            });
            actions.push(UiAction::WaitText {
                text: "Create Extension",
                timeout_ms: 10_000,
            });
            actions
        }
        JourneyId::ConfigureInstalledExtension => {
            let mut actions = open_extension_manager_installed_tab();
            actions.push(UiAction::WaitText {
                text: "CPU usage",
                timeout_ms: 60_000,
            });
            actions.push(UiAction::Expect { text: "Restart" });
            actions
        }
        JourneyId::UninstallExtension => {
            let mut actions = open_extension_manager_installed_tab();
            actions.push(UiAction::WaitText {
                text: "Uninstall",
                timeout_ms: 60_000,
            });
            actions
        }
        JourneyId::AddCustomManifest => {
            let mut actions = open_extension_manager();
            actions.push(UiAction::ClickSelector {
                css: ".v-main .mdi-cog",
            });
            actions.push(UiAction::WaitText {
                text: "Extensions Manifest",
                timeout_ms: 10_000,
            });
            actions.push(UiAction::Expect {
                text: "Top ones have higher priority",
            });
            actions
        }
        JourneyId::EditExtensionDevVersion => {
            let mut actions = open_extension_manager_installed_tab();
            actions.push(UiAction::WaitText {
                text: "Edit",
                timeout_ms: 60_000,
            });
            actions.push(UiAction::Click { text: "Edit" });
            actions.push(UiAction::WaitText {
                text: "Extension Identifier",
                timeout_ms: 10_000,
            });
            actions
        }
        JourneyId::UpdateBlueosVersion => vec![
            UiAction::Open {
                path: "/tools/version-chooser",
            },
            UiAction::WaitText {
                text: "Current Version",
                timeout_ms: 60_000,
            },
            UiAction::Expect {
                text: "Pirate mode",
            },
        ],
        JourneyId::SwitchLocalBlueosVersion => vec![
            UiAction::Open {
                path: "/tools/version-chooser",
            },
            UiAction::WaitText {
                text: "Local Versions",
                timeout_ms: 60_000,
            },
        ],
        JourneyId::PullBlueosVersionWithoutSwitch => {
            let mut actions = enable_pirate_mode_if_needed();
            actions.push(UiAction::Open {
                path: "/tools/version-chooser",
            });
            actions.push(UiAction::WaitText {
                text: "Remote Versions",
                timeout_ms: 60_000,
            });
            actions.push(UiAction::Expect {
                text: "Remote repository",
            });
            actions
        }
        JourneyId::DockerRegistryLogin => {
            let mut actions = enable_pirate_mode_if_needed();
            actions.push(UiAction::Open {
                path: "/tools/version-chooser",
            });
            actions.push(UiAction::WaitText {
                text: "Docker Login",
                timeout_ms: 60_000,
            });
            actions.push(UiAction::Click {
                text: "Docker Login",
            });
            actions.push(UiAction::WaitText {
                text: "Custom Registry",
                timeout_ms: 10_000,
            });
            actions
        }
        JourneyId::RenameVehicle => vec![
            UiAction::Open { path: "/" },
            UiAction::HoverSelector {
                css: "#vehicle-name",
            },
            UiAction::ClickSelector { css: ".edit-icon" },
            UiAction::WaitText {
                text: "Edit Vehicle Details",
                timeout_ms: 10_000,
            },
            UiAction::Expect {
                text: "Vehicle Name",
            },
        ],
        JourneyId::ModifyBagDatabase => vec![
            UiAction::Open {
                path: "/tools/bag-editor",
            },
            UiAction::WaitText {
                text: "powered by ace",
                timeout_ms: 60_000,
            },
            UiAction::WaitText {
                text: "wizard",
                timeout_ms: 60_000,
            },
        ],
        JourneyId::ViewConfiguredNmeaSockets => vec![
            UiAction::Open {
                path: "/tools/nmea-injector",
            },
            UiAction::WaitText {
                text: "The NMEA Injector receives NMEA via UDP or TCP",
                timeout_ms: 60_000,
            },
            UiAction::Expect {
                text: "No NMEA sockets available",
            },
        ],
        JourneyId::AddExternalNmeaGpsSocket => {
            let mut actions = open_nmea_injector();
            actions.extend(nmea_socket_creation_dialog_plan());
            actions
        }
        JourneyId::RemoveConfiguredNmeaSocket => vec![
            UiAction::Open {
                path: "/tools/nmea-injector",
            },
            UiAction::WaitText {
                text: "UDP:9999",
                timeout_ms: 60_000,
            },
            UiAction::ClickSelector {
                css: ".injector-remove-btn",
            },
            UiAction::ExpectGone {
                text: "UDP:9999",
                timeout_ms: 30_000,
            },
        ],
        JourneyId::CreateSerialToUdpBridge => {
            let mut actions = open_serial_bridges();
            actions.extend(serial_bridge_creation_dialog_plan());
            actions
        }
        JourneyId::ChangeMdnsHostname => vec![
            UiAction::Open { path: "/" },
            UiAction::HoverSelector {
                css: "#vehicle-name",
            },
            UiAction::ClickSelector { css: ".edit-icon" },
            UiAction::WaitText {
                text: "Edit Vehicle Details",
                timeout_ms: 10_000,
            },
            UiAction::Expect {
                text: "mDNS Hostname",
            },
            UiAction::Click { text: "Cancel" },
        ],
        JourneyId::AssignStaticIpAddress => {
            let mut actions = open_ethernet_tray();
            actions.push(expand_ethernet_interface("eth0"));
            actions.extend(static_ip_dialog_landmarks());
            actions.push(UiAction::PressKey { key: "Escape" });
            actions.push(UiAction::ExpectGone {
                text: "New static IP address",
                timeout_ms: 10_000,
            });
            actions.push(close_ethernet_tray());
            actions
        }
        JourneyId::AcquireDynamicIpAddress => {
            let mut actions = open_ethernet_tray();
            actions.push(expand_ethernet_interface("eth0"));
            actions.push(UiAction::Expect {
                text: "Ask for dynamic IP",
            });
            actions.push(close_ethernet_tray());
            actions
        }
        JourneyId::EnableOnboardDhcpServer => {
            let mut actions = open_ethernet_tray();
            actions.push(expand_ethernet_interface("eth0"));
            actions.push(UiAction::Expect {
                text: "Enable DHCP server",
            });
            actions.push(close_ethernet_tray());
            actions
        }
        JourneyId::DisableOnboardDhcpServer => {
            let mut actions = open_ethernet_tray();
            actions.push(expand_ethernet_interface("eth0"));
            actions.push(UiAction::Expect {
                text: "Disable DHCP server",
            });
            actions.push(close_ethernet_tray());
            actions
        }
        JourneyId::SetNetworkInterfacePriority => {
            let mut actions = enable_pirate_mode_if_needed();
            actions.extend(open_internet_network_menu());
            actions.push(UiAction::Expect {
                text: "Applied changes require a",
            });
            actions.push(UiAction::Click { text: "Cancel" });
            actions.extend(close_internet_network_menu());
            actions
        }
        JourneyId::ConfigureHostDns => {
            let mut actions = enable_pirate_mode_if_needed();
            actions.extend(open_internet_network_menu());
            actions.push(UiAction::Click {
                text: "Dns Configuration",
            });
            actions.push(UiAction::WaitText {
                text: "Host DNS nameservers",
                timeout_ms: 10_000,
            });
            actions.push(UiAction::Expect {
                text: "Lock DNS nameservers configuration",
            });
            actions.push(UiAction::Click {
                text: "Host DNS nameservers",
            });
            actions.extend(close_internet_network_menu());
            actions
        }
        JourneyId::ConnectToWifiNetwork => {
            let mut actions = open_wifi_tray();
            actions.extend(wifi_scan_network_dialog_landmarks());
            actions.push(close_wifi_tray());
            actions
        }
        JourneyId::ConnectToHiddenWifiNetwork => {
            let mut actions = open_wifi_tray();
            actions.push(UiAction::ClickIfVisible {
                text: "[HIDDEN SSID]",
            });
            actions.push(UiAction::WaitText {
                text: "SSID",
                timeout_ms: 10_000,
            });
            actions.push(UiAction::Expect { text: "Connect" });
            actions.push(close_wifi_tray());
            actions
        }
        JourneyId::DisconnectFromWifiNetwork => {
            let mut actions = open_wifi_tray();
            actions.push(UiAction::ClickSelector {
                css: ".network-card:not(.available-network)",
            });
            actions.push(UiAction::WaitText {
                text: "Disconnect",
                timeout_ms: 10_000,
            });
            actions.push(UiAction::Expect { text: "Disconnect" });
            actions.push(close_wifi_tray());
            actions
        }
        JourneyId::ForgetSavedWifiNetwork => {
            let mut actions = open_wifi_tray();
            actions.extend(wifi_saved_network_dialog_landmarks());
            actions.push(UiAction::WaitText {
                text: "Forget",
                timeout_ms: 10_000,
            });
            actions.push(UiAction::Expect { text: "Forget" });
            actions.push(close_wifi_tray());
            actions
        }
        JourneyId::ForceWifiNetworkPassword => {
            let mut actions = open_wifi_tray();
            actions.extend(wifi_saved_network_dialog_landmarks());
            actions.push(UiAction::WaitText {
                text: "Force new password",
                timeout_ms: 10_000,
            });
            actions.push(UiAction::Click {
                text: "Force new password",
            });
            actions.push(UiAction::Expect { text: "Password" });
            actions.push(close_wifi_tray());
            actions
        }
        JourneyId::ReconnectToSavedWifiNetwork => {
            let mut actions = open_wifi_tray();
            actions.extend(wifi_saved_network_dialog_landmarks());
            actions.push(UiAction::Expect { text: "Connect" });
            actions.push(close_wifi_tray());
            actions
        }
        JourneyId::RejectInvalidWifiCredentials => {
            let mut actions = open_wifi_tray();
            actions.extend(wifi_scan_network_dialog_landmarks());
            actions.push(UiAction::WaitText {
                text: "Password",
                timeout_ms: 10_000,
            });
            actions.push(UiAction::Expect { text: "Connect" });
            actions.push(close_wifi_tray());
            actions
        }
        JourneyId::DetectWifiApLoss => {
            let mut actions = open_wifi_tray();
            actions.push(UiAction::Sleep { ms: 2000 });
            actions.push(UiAction::Expect { text: "Wifi" });
            actions
        }
        JourneyId::AutoconnectToSavedWifiNetwork => {
            let mut actions = open_wifi_tray();
            actions.push(UiAction::Sleep { ms: 2000 });
            actions.push(UiAction::Expect { text: "Wifi" });
            actions
        }
        JourneyId::ToggleHotspot => {
            let mut actions = open_wifi_tray();
            actions.push(UiAction::Expect { text: "Wifi" });
            actions
        }
        JourneyId::ConfigureHotspotCredentials => {
            let mut actions = open_wifi_settings();
            actions.extend(wifi_settings_hotspot_landmarks());
            actions.push(close_wifi_tray());
            actions
        }
        JourneyId::ToggleSmartHotspot => {
            let mut actions = open_wifi_settings();
            actions.push(UiAction::Expect {
                text: "Enable smart-hotspot",
            });
            actions.push(close_wifi_tray());
            actions
        }
        JourneyId::RunLanSpeedTest => vec![
            UiAction::Open {
                path: "/tools/network-test",
            },
            UiAction::WaitText {
                text: "Local network test",
                timeout_ms: 60_000,
            },
            UiAction::Expect {
                text: "Local Network Speed and Latency Test",
            },
        ],
        _ => return None,
    };
    Some(UiJourneyPlan {
        journey_id: id.as_str().to_string(),
        sitl_frame: if matches!(id, JourneyId::DetectMotorDirections) {
            Some(SITL_FRAME_VECTORED)
        } else if UI_CALIBRATION_JOURNEYS.contains(&id) {
            Some(SITL_FRAME_CALIBRATION)
        } else {
            None
        },
        actions,
    })
}

pub fn ui_suite_plans() -> Vec<UiJourneyPlan> {
    UI_NO_HARDWARE_JOURNEYS
        .iter()
        .chain(UI_CALIBRATION_JOURNEYS.iter())
        .copied()
        .filter_map(ui_plan)
        .collect()
}

fn open_extension_manager() -> Vec<UiAction> {
    vec![
        UiAction::Open {
            path: "/tools/extensions-manager",
        },
        UiAction::WaitText {
            text: "Installed",
            timeout_ms: 60_000,
        },
    ]
}

fn open_extension_manager_store_tab() -> Vec<UiAction> {
    let mut actions = open_extension_manager();
    actions.push(UiAction::ClickIfVisible { text: "Store" });
    actions.push(UiAction::ClickIfVisible { text: "Back Alley" });
    actions
}

fn open_extension_manager_installed_tab() -> Vec<UiAction> {
    let mut actions = open_extension_manager();
    actions.push(UiAction::Click { text: "Installed" });
    actions
}

fn stream_creation_dialog_plan() -> Vec<UiAction> {
    vec![
        UiAction::Click { text: "Add stream" },
        UiAction::WaitText {
            text: "Stream creation",
            timeout_ms: 10_000,
        },
        UiAction::Expect {
            text: "Stream nickname",
        },
        UiAction::Expect { text: "Encoding" },
        UiAction::Click { text: "Cancel" },
    ]
}

fn open_nmea_injector() -> Vec<UiAction> {
    vec![
        UiAction::Open {
            path: "/tools/nmea-injector",
        },
        UiAction::WaitText {
            text: "The NMEA Injector receives NMEA via UDP or TCP",
            timeout_ms: 60_000,
        },
    ]
}

fn nmea_socket_creation_dialog_plan() -> Vec<UiAction> {
    vec![
        UiAction::ClickSelector {
            css: ".v-btn--example",
        },
        UiAction::WaitText {
            text: "New NMEA socket",
            timeout_ms: 10_000,
        },
        UiAction::Expect {
            text: "Socket kind",
        },
        UiAction::Expect {
            text: "Socket port",
        },
        UiAction::Expect {
            text: "Mavlink component ID",
        },
        UiAction::Click { text: "Cancel" },
    ]
}

fn open_wifi_tray() -> Vec<UiAction> {
    vec![
        UiAction::Open { path: "/" },
        UiAction::ClickSelector {
            css: "#wifi-tray-menu-button",
        },
        UiAction::WaitText {
            text: "Wifi",
            timeout_ms: 60_000,
        },
    ]
}

fn close_wifi_tray() -> UiAction {
    UiAction::ClickSelector {
        css: "#wifi-tray-menu-button",
    }
}

fn open_wifi_settings() -> Vec<UiAction> {
    let mut actions = open_wifi_tray();
    actions.push(UiAction::ClickSelector {
        css: ".v-menu__content .mdi-cog",
    });
    actions.push(UiAction::WaitText {
        text: "Wifi settings",
        timeout_ms: 10_000,
    });
    actions
}

fn wifi_scan_network_dialog_landmarks() -> Vec<UiAction> {
    vec![
        UiAction::ClickSelector {
            css: ".available-network",
        },
        UiAction::WaitText {
            text: "Connect",
            timeout_ms: 10_000,
        },
        UiAction::Expect { text: "Connect" },
    ]
}

fn wifi_saved_network_dialog_landmarks() -> Vec<UiAction> {
    vec![
        UiAction::ClickSelector {
            css: ".available-network",
        },
        UiAction::WaitText {
            text: "Connect",
            timeout_ms: 10_000,
        },
    ]
}

fn wifi_settings_hotspot_landmarks() -> Vec<UiAction> {
    vec![
        UiAction::Expect {
            text: "Hotspot SSID",
        },
        UiAction::Expect {
            text: "Hotspot password",
        },
    ]
}

fn open_ethernet_tray() -> Vec<UiAction> {
    vec![
        UiAction::Open { path: "/" },
        UiAction::ClickSelector {
            css: "#ethernet-tray-menu-button",
        },
        UiAction::WaitText {
            text: "Connected",
            timeout_ms: 60_000,
        },
    ]
}

fn close_ethernet_tray() -> UiAction {
    UiAction::ClickSelector {
        css: "#ethernet-tray-menu-button",
    }
}

fn expand_ethernet_interface(name: &'static str) -> UiAction {
    let _ = name;
    UiAction::ClickSelector {
        css: ".v-expansion-panel-header:has-text(\"eth0\")",
    }
}

fn static_ip_dialog_landmarks() -> Vec<UiAction> {
    vec![
        UiAction::ClickSelector {
            css: ".v-expansion-panel-content .v-btn:has-text(\"static IP\")",
        },
        UiAction::WaitText {
            text: "New static IP address",
            timeout_ms: 10_000,
        },
        UiAction::Expect { text: "IP address" },
    ]
}

fn enable_pirate_mode_if_needed() -> Vec<UiAction> {
    vec![
        UiAction::Open { path: "/" },
        UiAction::ClickSelector {
            css: ".v-app-bar .v-icon[class*=\"mdi-robot-happy\"], .v-app-bar .v-icon[class*=\"mdi-skull-crossbones\"]",
        },
        UiAction::ClickIfVisible {
            text: "Enable Pirate Mode",
        },
        UiAction::Sleep { ms: 500 },
        UiAction::WaitText {
            text: "Bag Editor",
            timeout_ms: 30_000,
        },
    ]
}

fn open_internet_network_menu() -> Vec<UiAction> {
    vec![
        UiAction::Open { path: "/" },
        UiAction::ClickSelector {
            css: ".v-app-bar .v-icon[class*=\"mdi-web\"]",
        },
        UiAction::WaitText {
            text: "Network Interface Priority",
            timeout_ms: 30_000,
        },
    ]
}

fn close_internet_network_menu() -> Vec<UiAction> {
    vec![
        UiAction::PressKey { key: "Escape" },
        UiAction::ExpectGone {
            text: "Network Interface Priority",
            timeout_ms: 10_000,
        },
    ]
}

fn open_serial_bridges() -> Vec<UiAction> {
    vec![
        UiAction::Open {
            path: "/tools/bridges",
        },
        UiAction::WaitText {
            text: "No bridges available",
            timeout_ms: 60_000,
        },
    ]
}

fn serial_bridge_creation_dialog_plan() -> Vec<UiAction> {
    vec![
        UiAction::ClickSelector {
            css: ".v-btn--example",
        },
        UiAction::WaitText {
            text: "New bridge",
            timeout_ms: 10_000,
        },
        UiAction::Expect {
            text: "Serial baudrate",
        },
        UiAction::Expect {
            text: "Server Mode",
        },
        UiAction::Click { text: "Cancel" },
    ]
}

fn open_configure(subtab: &'static str) -> UiAction {
    match subtab {
        "gyroscope" => UiAction::Open {
            path: "/vehicle/setup/configure/gyroscope",
        },
        "baro" => UiAction::Open {
            path: "/vehicle/setup/configure/baro",
        },
        "accelerometer" => UiAction::Open {
            path: "/vehicle/setup/configure/accelerometer",
        },
        "compass" => UiAction::Open {
            path: "/vehicle/setup/configure/compass",
        },
        _ => UiAction::Open {
            path: "/vehicle/setup/configure",
        },
    }
}

fn sitl_rc(rc: SitlRc) -> UiAction {
    UiAction::SitlRc {
        chan5: rc.chan5,
        chan6: rc.chan6,
        chan7: rc.chan7,
        chan8: rc.chan8,
    }
}

fn accel_plan() -> Vec<UiAction> {
    let poses: [(&str, i16, i16, i16); 6] = [
        ("Place the vehicle on a level surface", 0, 0, 0),
        ("Place the vehicle on its left side", -90, 0, 0),
        ("Place the vehicle on its right side", 90, 0, 0),
        ("Place the vehicle with its nose down", 0, 90, 0),
        ("Place the vehicle with its nose up", 0, -90, 0),
        ("Place the vehicle on its back", 0, 180, 0),
    ];
    let mut actions = vec![
        open_configure("accelerometer"),
        sitl_rc(SitlRc::attitude_deg(0, 0, 0)),
        UiAction::Sleep { ms: 2500 },
        UiAction::Click {
            text: "Start Full Calibration",
        },
        UiAction::Click {
            text: "Start Calibration",
        },
    ];
    for (i, (prompt, roll, pitch, yaw)) in poses.iter().enumerate() {
        if i > 0 {
            actions.push(sitl_rc(SitlRc::attitude_deg(*roll, *pitch, *yaw)));
        }
        actions.push(UiAction::WaitText {
            text: prompt,
            timeout_ms: 30_000,
        });
        actions.push(UiAction::Sleep { ms: 1500 });
        actions.push(UiAction::Click { text: "Next" });
        actions.push(UiAction::Sleep { ms: 2000 });
    }
    actions.push(UiAction::WaitText {
        text: "Calibrated",
        timeout_ms: 30_000,
    });
    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_journeys_all_have_plans() {
        for id in UI_CALIBRATION_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert_eq!(plan.journey_id, id.as_str());
            assert!(!plan.actions.is_empty());
            assert!(plan.sitl_frame.is_some());
        }
    }

    #[test]
    fn gyro_plan_opens_configure_and_stops_sitl() {
        let plan = ui_plan(JourneyId::CalibrateGyroscope).unwrap();
        assert!(matches!(
            &plan.actions[0],
            UiAction::Open {
                path: "/vehicle/setup/configure/gyroscope"
            }
        ));
        assert!(plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::SitlRc { .. })));
        assert_eq!(plan.sitl_frame, Some(SITL_FRAME_CALIBRATION));
    }

    #[test]
    fn accel_plan_has_six_attitudes() {
        let plan = ui_plan(JourneyId::CalibrateAccelerometer).unwrap();
        let attitudes = plan
            .actions
            .iter()
            .filter(
                |a| matches!(a, UiAction::SitlRc { chan5, .. } if *chan5 >= 1100 && *chan5 < 1200),
            )
            .count();
        assert_eq!(attitudes, 6);
    }

    #[test]
    fn motor_detect_uses_vectored() {
        let plan = ui_plan(JourneyId::DetectMotorDirections).unwrap();
        assert_eq!(plan.sitl_frame, Some(SITL_FRAME_VECTORED));
    }

    #[test]
    fn compass_plan_tries_geoip_if_present() {
        let plan = ui_plan(JourneyId::CalibrateCompass).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::ClickIfVisible {
                text: "Use GeoIP coordinates"
            }
        )));
    }

    #[test]
    fn no_hardware_plans_have_no_sitl_frame() {
        for id in UI_NO_HARDWARE_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert!(plan.sitl_frame.is_none(), "{id}");
            assert!(!plan.actions.is_empty());
        }
    }

    #[test]
    fn unknown_journey_has_no_plan() {
        assert!(ui_plan(JourneyId::MonitorInternetConnectivity).is_none());
    }

    #[test]
    fn extension_journeys_all_have_plans() {
        for id in UI_EXTENSION_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert_eq!(plan.journey_id, id.as_str());
            assert!(plan.sitl_frame.is_none(), "{id}");
            assert!(!plan.actions.is_empty());
        }
    }

    #[test]
    fn version_settings_journeys_all_have_plans() {
        for id in UI_VERSION_SETTINGS_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert_eq!(plan.journey_id, id.as_str());
            assert!(plan.sitl_frame.is_none(), "{id}");
            assert!(!plan.actions.is_empty());
        }
    }

    #[test]
    fn typed_skip_covers_orchestrated_without_frontend_actor() {
        use crate::catalog::Catalog;
        use crate::journey::{derive_oracle_class, OracleClass};
        use crate::journey_matrix::{has_frontend_step, PAGE_LOAD_UI};

        let catalog = Catalog::bootstrap();
        for journey in catalog.journeys() {
            if derive_oracle_class(&catalog, journey) != OracleClass::ClientOrchestrated {
                continue;
            }
            if has_frontend_step(journey) {
                continue;
            }
            let covered = ui_plan(journey.id).is_some()
                || PAGE_LOAD_UI.contains(&journey.id)
                || ui_typed_skip_reason(journey.id).is_some();
            assert!(
                covered,
                "orchestrated without frontend actor: {}",
                journey.id
            );
        }
    }

    #[test]
    fn rename_vehicle_plan_opens_edit_dialog() {
        let plan = ui_plan(JourneyId::RenameVehicle).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::Expect {
                text: "Vehicle Name"
            }
        )));
    }

    #[test]
    fn bag_journeys_all_have_plans() {
        for id in UI_BAG_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert_eq!(plan.journey_id, id.as_str());
            assert!(plan.sitl_frame.is_none(), "{id}");
            assert!(!plan.actions.is_empty());
        }
    }

    #[test]
    fn modify_bag_database_plan_loads_json_editor() {
        let plan = ui_plan(JourneyId::ModifyBagDatabase).unwrap();
        assert!(matches!(
            &plan.actions[0],
            UiAction::Open {
                path: "/tools/bag-editor"
            }
        ));
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::WaitText {
                text: "powered by ace",
                ..
            }
        )));
        assert!(plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::WaitText { text: "wizard", .. })));
    }

    #[test]
    fn camera_plans_exist_without_sitl_frame() {
        for id in UI_CAMERA_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert_eq!(plan.journey_id, id.as_str());
            assert!(plan.sitl_frame.is_none(), "{id}");
            assert!(!plan.actions.is_empty());
        }
    }

    #[test]
    fn remove_camera_stream_clicks_remove_btn() {
        let plan = ui_plan(JourneyId::RemoveCameraStream).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::ClickSelector {
                css: ".stream-remove-btn"
            }
        )));
    }

    #[test]
    fn configure_video_stream_resets_settings_without_legacy_toggle() {
        let plan = ui_plan(JourneyId::ConfigureVideoStream).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::Click {
                text: "Reset Settings"
            }
        )));
        let legacy = plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Click { text } if text.contains("legacy")));
        assert!(!legacy);
    }

    #[test]
    fn nmea_journeys_all_have_plans() {
        for id in UI_NMEA_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert_eq!(plan.journey_id, id.as_str());
            assert!(plan.sitl_frame.is_none(), "{id}");
            assert!(!plan.actions.is_empty());
        }
    }

    #[test]
    fn add_nmea_socket_plan_opens_dialog_and_cancels() {
        let plan = ui_plan(JourneyId::AddExternalNmeaGpsSocket).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::WaitText {
                text: "New NMEA socket",
                ..
            }
        )));
        assert!(plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Click { text: "Cancel" })));
        assert!(!plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Click { text: "Create" })));
    }

    #[test]
    fn remove_nmea_socket_clicks_remove_btn() {
        let plan = ui_plan(JourneyId::RemoveConfiguredNmeaSocket).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::ClickSelector {
                css: ".injector-remove-btn"
            }
        )));
    }

    #[test]
    fn bridget_journeys_all_have_plans() {
        for id in UI_BRIDGET_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert_eq!(plan.journey_id, id.as_str());
            assert!(plan.sitl_frame.is_none(), "{id}");
            assert!(!plan.actions.is_empty());
        }
    }

    #[test]
    fn create_serial_bridge_plan_opens_dialog_and_cancels() {
        let plan = ui_plan(JourneyId::CreateSerialToUdpBridge).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::WaitText {
                text: "New bridge",
                ..
            }
        )));
        assert!(plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Click { text: "Cancel" })));
        assert!(!plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Click { text: "Create" })));
    }

    #[test]
    fn http_passthrough_internet_journeys_are_typed_skip() {
        for id in [
            JourneyId::AccessBlueosWebInterface,
            JourneyId::DiscoverBlueosOnNetwork,
            JourneyId::MonitorInternetConnectivity,
            JourneyId::ProbeInterfaceInternetConnectivity,
            JourneyId::VerifyInternetConnectivity,
        ] {
            assert_eq!(
                ui_typed_skip_reason(id),
                Some("http_is_operator_contract"),
                "{id}"
            );
            assert!(ui_plan(id).is_none(), "{id}");
        }
    }

    #[test]
    fn remove_serial_bridge_is_usb_serial_typed_skip() {
        assert_eq!(
            ui_typed_skip_reason(JourneyId::RemoveSerialBridge),
            Some("usb_serial_device")
        );
        assert!(ui_plan(JourneyId::RemoveSerialBridge).is_none());
    }

    #[test]
    fn cable_guy_journeys_all_have_plans() {
        for id in UI_CABLE_GUY_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert_eq!(plan.journey_id, id.as_str());
            assert!(plan.sitl_frame.is_none(), "{id}");
            assert!(!plan.actions.is_empty());
        }
    }

    #[test]
    fn change_mdns_hostname_plan_expects_mdns_field_and_cancels() {
        let plan = ui_plan(JourneyId::ChangeMdnsHostname).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::Expect {
                text: "mDNS Hostname"
            }
        )));
        assert!(plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Click { text: "Cancel" })));
        assert!(!plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Click { text: "Save" })));
    }

    #[test]
    fn pirate_gated_cable_guy_plans_enable_pirate_mode_before_network_menu() {
        for id in [
            JourneyId::ConfigureHostDns,
            JourneyId::SetNetworkInterfacePriority,
        ] {
            let plan = ui_plan(id).unwrap();
            let actions = &plan.actions;
            let pirate_tray = actions.iter().position(|a| {
                matches!(
                    a,
                    UiAction::ClickSelector { css }
                    if css.contains("mdi-robot-happy") && css.contains("mdi-skull-crossbones")
                )
            });
            assert!(
                !actions.iter().any(|a| matches!(
                    a,
                    UiAction::ClickSelector { css } if css.contains("pirate-mode-tray-menu-button")
                )),
                "{id}: must not use shared #pirate-mode-tray-menu-button id"
            );
            let bag_editor = actions.iter().position(|a| {
                matches!(
                    a,
                    UiAction::WaitText {
                        text: "Bag Editor",
                        ..
                    }
                )
            });
            let network_priority = actions.iter().position(|a| {
                matches!(
                    a,
                    UiAction::WaitText {
                        text: "Network Interface Priority",
                        ..
                    }
                )
            });
            let (Some(pirate_tray), Some(bag_editor), Some(network_priority)) =
                (pirate_tray, bag_editor, network_priority)
            else {
                panic!("{id}: missing pirate tray, Bag Editor proof, or network menu wait");
            };
            assert!(
                pirate_tray < bag_editor && bag_editor < network_priority,
                "{id}: pirate enable must precede Bag Editor proof before network menu"
            );
        }
    }

    #[test]
    fn assign_static_ip_plan_opens_dialog_without_create() {
        let plan = ui_plan(JourneyId::AssignStaticIpAddress).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::WaitText {
                text: "New static IP address",
                ..
            }
        )));
        assert!(plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::PressKey { key: "Escape" })));
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::ExpectGone {
                text: "New static IP address",
                ..
            }
        )));
        assert!(!plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Click { text: "Create" })));
    }

    #[test]
    fn cable_guy_plans_do_not_apply_network_mutations() {
        let mutating_clicks = [
            "Create",
            "Enable",
            "APPLY",
            "Apply",
            "Disable",
            "Save",
            "Ask for dynamic IP",
        ];
        for id in UI_CABLE_GUY_JOURNEYS {
            if *id == JourneyId::ChangeMdnsHostname {
                continue;
            }
            let plan = ui_plan(*id).expect("plan");
            for label in mutating_clicks {
                assert!(
                    !plan
                        .actions
                        .iter()
                        .any(|a| matches!(a, UiAction::Click { text } if *text == label)),
                    "{id} must not click {label}"
                );
            }
        }
    }

    #[test]
    fn wifi_journeys_all_have_plans() {
        for id in UI_WIFI_JOURNEYS {
            let plan = ui_plan(*id).expect("plan");
            assert_eq!(plan.journey_id, id.as_str());
            assert!(plan.sitl_frame.is_none(), "{id}");
            assert!(!plan.actions.is_empty());
        }
    }

    #[test]
    fn connect_wifi_plan_opens_tray_and_does_not_connect() {
        let plan = ui_plan(JourneyId::ConnectToWifiNetwork).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::ClickSelector {
                css: "#wifi-tray-menu-button"
            }
        )));
        assert!(plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Expect { text: "Connect" })));
        assert!(!plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Click { text: "Connect" })));
    }

    #[test]
    fn toggle_hotspot_plan_does_not_click_hotspot_toggle() {
        let plan = ui_plan(JourneyId::ToggleHotspot).unwrap();
        let clicks_hotspot = plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::ClickSelector { css } if css.contains("access-point")));
        assert!(!clicks_hotspot);
    }

    #[test]
    fn pirate_gated_version_chooser_plans_enable_pirate_mode_before_landmark() {
        let cases = [
            (JourneyId::PullBlueosVersionWithoutSwitch, "Remote Versions"),
            (JourneyId::DockerRegistryLogin, "Docker Login"),
        ];
        for (id, landmark) in cases {
            let plan = ui_plan(id).unwrap();
            let actions = &plan.actions;
            let pirate_tray = actions.iter().position(|a| {
                matches!(
                    a,
                    UiAction::ClickSelector { css }
                    if css.contains("mdi-robot-happy") && css.contains("mdi-skull-crossbones")
                )
            });
            let bag_editor = actions.iter().position(|a| {
                matches!(
                    a,
                    UiAction::WaitText {
                        text: "Bag Editor",
                        ..
                    }
                )
            });
            let landmark_wait = actions.iter().position(|a| {
                matches!(
                    a,
                    UiAction::WaitText {
                        text,
                        ..
                    } if *text == landmark
                )
            });
            let (Some(pirate_tray), Some(bag_editor), Some(landmark_wait)) =
                (pirate_tray, bag_editor, landmark_wait)
            else {
                panic!("{id}: missing pirate tray, Bag Editor proof, or {landmark} wait");
            };
            assert!(
                pirate_tray < bag_editor && bag_editor < landmark_wait,
                "{id}: pirate enable must precede Bag Editor proof before {landmark}"
            );
        }
    }

    #[test]
    fn install_custom_extension_clicks_code_braces_if_visible() {
        let plan = ui_plan(JourneyId::InstallCustomExtension).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::ClickSelectorIfVisible {
                css: ".v-speed-dial--is-active .mdi-code-braces"
            }
        )));
        assert!(!plan.actions.iter().any(|a| matches!(
            a,
            UiAction::ClickSelector {
                css: ".v-speed-dial--is-active .mdi-code-braces"
            }
        )));
    }

    #[test]
    fn add_custom_manifest_scopes_settings_cog_to_main() {
        let plan = ui_plan(JourneyId::AddCustomManifest).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::ClickSelector {
                css: ".v-main .mdi-cog"
            }
        )));
        assert!(!plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::ClickSelector { css: ".mdi-cog" })));
    }

    #[test]
    fn configure_hotspot_credentials_plan_opens_settings_without_save() {
        let plan = ui_plan(JourneyId::ConfigureHotspotCredentials).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::WaitText {
                text: "Wifi settings",
                ..
            }
        )));
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            UiAction::Expect {
                text: "Hotspot SSID"
            }
        )));
        assert!(!plan
            .actions
            .iter()
            .any(|a| matches!(a, UiAction::Click { text: "Save" })));
    }
}
