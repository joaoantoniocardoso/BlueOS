use std::fmt;

use schemars::JsonSchema;
use serde::Serialize;

use super::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, JsonSchema)]
pub enum JourneyId {
    #[serde(rename = "access_blueos_web_interface")]
    AccessBlueosWebInterface,
    #[serde(rename = "access_web_terminal")]
    AccessWebTerminal,
    #[serde(rename = "acquire_dynamic_ip_address")]
    AcquireDynamicIpAddress,
    #[serde(rename = "add_custom_manifest")]
    AddCustomManifest,
    #[serde(rename = "add_external_nmea_gps_socket")]
    AddExternalNmeaGpsSocket,
    #[serde(rename = "apply_parameter_file")]
    ApplyParameterFile,
    #[serde(rename = "assign_static_ip_address")]
    AssignStaticIpAddress,
    #[serde(rename = "autoconnect_to_saved_wifi_network")]
    AutoconnectToSavedWifiNetwork,
    #[serde(rename = "browse_available_web_services")]
    BrowseAvailableWebServices,
    #[serde(rename = "browse_extension_store")]
    BrowseExtensionStore,
    #[serde(rename = "browse_video_recordings")]
    BrowseVideoRecordings,
    #[serde(rename = "calibrate_accelerometer")]
    CalibrateAccelerometer,
    #[serde(rename = "calibrate_barometer")]
    CalibrateBarometer,
    #[serde(rename = "calibrate_compass")]
    CalibrateCompass,
    #[serde(rename = "calibrate_gyroscope")]
    CalibrateGyroscope,
    #[serde(rename = "change_board")]
    ChangeBoard,
    #[serde(rename = "change_mdns_hostname")]
    ChangeMdnsHostname,
    #[serde(rename = "change_ui_theme_color")]
    ChangeUiThemeColor,
    #[serde(rename = "configure_camera_stream")]
    ConfigureCameraStream,
    #[serde(rename = "configure_host_dns")]
    ConfigureHostDns,
    #[serde(rename = "configure_hotspot_credentials")]
    ConfigureHotspotCredentials,
    #[serde(rename = "configure_installed_extension")]
    ConfigureInstalledExtension,
    #[serde(rename = "configure_uvc_device_controls")]
    ConfigureUvcDeviceControls,
    #[serde(rename = "configure_video_stream")]
    ConfigureVideoStream,
    #[serde(rename = "connect_ping_viewer_to_sonar")]
    ConnectPingViewerToSonar,
    #[serde(rename = "connect_to_wifi_network")]
    ConnectToWifiNetwork,
    #[serde(rename = "connect_to_hidden_wifi_network")]
    ConnectToHiddenWifiNetwork,
    #[serde(rename = "create_serial_to_udp_bridge")]
    CreateSerialToUdpBridge,
    #[serde(rename = "delete_3d_model_override")]
    Delete3dModelOverride,
    #[serde(rename = "delete_local_blueos_version")]
    DeleteLocalBlueosVersion,
    #[serde(rename = "delete_video_recording")]
    DeleteVideoRecording,
    #[serde(rename = "deploy")]
    Deploy,
    #[serde(rename = "detect_motor_directions")]
    DetectMotorDirections,
    #[serde(rename = "detect_wifi_ap_loss")]
    DetectWifiApLoss,
    #[serde(rename = "disable_onboard_dhcp_server")]
    DisableOnboardDhcpServer,
    #[serde(rename = "disconnect_from_wifi_network")]
    DisconnectFromWifiNetwork,
    #[serde(rename = "discover_blueos_on_network")]
    DiscoverBlueosOnNetwork,
    #[serde(rename = "docker_registry_login")]
    DockerRegistryLogin,
    #[serde(rename = "download_video_recording")]
    DownloadVideoRecording,
    #[serde(rename = "edit_extension_dev_version")]
    EditExtensionDevVersion,
    #[serde(rename = "enable_legacy_camera_support")]
    EnableLegacyCameraSupport,
    #[serde(rename = "enable_onboard_dhcp_server")]
    EnableOnboardDhcpServer,
    #[serde(rename = "enable_ping1d_rangefinder_mavlink")]
    EnablePing1dRangefinderMavlink,
    #[serde(rename = "forget_saved_wifi_network")]
    ForgetSavedWifiNetwork,
    #[serde(rename = "force_wifi_network_password")]
    ForceWifiNetworkPassword,
    #[serde(rename = "free_disk_space")]
    FreeDiskSpace,
    #[serde(rename = "inspect_disk_usage")]
    InspectDiskUsage,
    #[serde(rename = "inspect_mavlink_messages_in_browser")]
    InspectMavlinkMessagesInBrowser,
    #[serde(rename = "inspect_raspberry_eeprom_bootloader")]
    InspectRaspberryEepromBootloader,
    #[serde(rename = "inspect_zenoh_network")]
    InspectZenohNetwork,
    #[serde(rename = "install_custom_extension")]
    InstallCustomExtension,
    #[serde(rename = "install_extension")]
    InstallExtension,
    #[serde(rename = "level_horizon")]
    LevelHorizon,
    #[serde(rename = "manage_blueos_files")]
    ManageBlueosFiles,
    #[serde(rename = "modify_bag_database")]
    ModifyBagDatabase,
    #[serde(rename = "monitor_internet_connectivity")]
    MonitorInternetConnectivity,
    #[serde(rename = "probe_interface_internet_connectivity")]
    ProbeInterfaceInternetConnectivity,
    #[serde(rename = "pull_blueos_version_without_switch")]
    PullBlueosVersionWithoutSwitch,
    #[serde(rename = "reboot_onboard_computer")]
    RebootOnboardComputer,
    #[serde(rename = "reconnect_to_saved_wifi_network")]
    ReconnectToSavedWifiNetwork,
    #[serde(rename = "reject_invalid_wifi_credentials")]
    RejectInvalidWifiCredentials,
    #[serde(rename = "remove_camera_stream")]
    RemoveCameraStream,
    #[serde(rename = "remove_configured_nmea_socket")]
    RemoveConfiguredNmeaSocket,
    #[serde(rename = "remove_custom_logo")]
    RemoveCustomLogo,
    #[serde(rename = "remove_custom_vehicle_image")]
    RemoveCustomVehicleImage,
    #[serde(rename = "remove_serial_bridge")]
    RemoveSerialBridge,
    #[serde(rename = "rename_vehicle")]
    RenameVehicle,
    #[serde(rename = "reset_blueos_settings")]
    ResetBlueosSettings,
    #[serde(rename = "reset_ui_theme_color")]
    ResetUiThemeColor,
    #[serde(rename = "restart_autopilot")]
    RestartAutopilot,
    #[serde(rename = "restore_default_firmware")]
    RestoreDefaultFirmware,
    #[serde(rename = "run_host_command")]
    RunHostCommand,
    #[serde(rename = "run_internet_speed_test")]
    RunInternetSpeedTest,
    #[serde(rename = "run_lan_speed_test")]
    RunLanSpeedTest,
    #[serde(rename = "run_multi_size_disk_speed_test")]
    RunMultiSizeDiskSpeedTest,
    #[serde(rename = "run_single_disk_speed_test")]
    RunSingleDiskSpeedTest,
    #[serde(rename = "run_sitl_simulation")]
    RunSitlSimulation,
    #[serde(rename = "set_network_interface_priority")]
    SetNetworkInterfacePriority,
    #[serde(rename = "shutdown_onboard_computer")]
    ShutdownOnboardComputer,
    #[serde(rename = "start_autopilot")]
    StartAutopilot,
    #[serde(rename = "stop_autopilot")]
    StopAutopilot,
    #[serde(rename = "switch_local_blueos_version")]
    SwitchLocalBlueosVersion,
    #[serde(rename = "sync_system_time")]
    SyncSystemTime,
    #[serde(rename = "toggle_hotspot")]
    ToggleHotspot,
    #[serde(rename = "toggle_smart_hotspot")]
    ToggleSmartHotspot,
    #[serde(rename = "uninstall_extension")]
    UninstallExtension,
    #[serde(rename = "update_blueos_version")]
    UpdateBlueosVersion,
    #[serde(rename = "update_bootstrap_image")]
    UpdateBootstrapImage,
    #[serde(rename = "update_firmware_online")]
    UpdateFirmwareOnline,
    #[serde(rename = "update_raspberry_eeprom_bootloader")]
    UpdateRaspberryEepromBootloader,
    #[serde(rename = "upload_3d_model_override")]
    Upload3dModelOverride,
    #[serde(rename = "upload_custom_firmware")]
    UploadCustomFirmware,
    #[serde(rename = "upload_custom_logo")]
    UploadCustomLogo,
    #[serde(rename = "upload_custom_vehicle_image")]
    UploadCustomVehicleImage,
    #[serde(rename = "vehicle_first_boot")]
    VehicleFirstBoot,
    #[serde(rename = "verify_internet_connectivity")]
    VerifyInternetConnectivity,
    #[serde(rename = "view_camera_streams")]
    ViewCameraStreams,
    #[serde(rename = "view_configured_nmea_sockets")]
    ViewConfiguredNmeaSockets,
    #[serde(rename = "view_configured_serial_bridges")]
    ViewConfiguredSerialBridges,
    #[serde(rename = "view_detected_sonar_devices")]
    ViewDetectedSonarDevices,
    #[serde(rename = "view_system_information")]
    ViewSystemInformation,
}

impl JourneyId {
    pub const ALL: [JourneyId; 101] = [
        JourneyId::AccessBlueosWebInterface,
        JourneyId::AccessWebTerminal,
        JourneyId::AcquireDynamicIpAddress,
        JourneyId::AddCustomManifest,
        JourneyId::AddExternalNmeaGpsSocket,
        JourneyId::ApplyParameterFile,
        JourneyId::AssignStaticIpAddress,
        JourneyId::AutoconnectToSavedWifiNetwork,
        JourneyId::BrowseAvailableWebServices,
        JourneyId::BrowseExtensionStore,
        JourneyId::BrowseVideoRecordings,
        JourneyId::CalibrateAccelerometer,
        JourneyId::CalibrateBarometer,
        JourneyId::CalibrateCompass,
        JourneyId::CalibrateGyroscope,
        JourneyId::ChangeBoard,
        JourneyId::ChangeMdnsHostname,
        JourneyId::ChangeUiThemeColor,
        JourneyId::ConfigureCameraStream,
        JourneyId::ConfigureHostDns,
        JourneyId::ConfigureHotspotCredentials,
        JourneyId::ConfigureInstalledExtension,
        JourneyId::ConfigureUvcDeviceControls,
        JourneyId::ConfigureVideoStream,
        JourneyId::ConnectPingViewerToSonar,
        JourneyId::ConnectToWifiNetwork,
        JourneyId::ConnectToHiddenWifiNetwork,
        JourneyId::CreateSerialToUdpBridge,
        JourneyId::Delete3dModelOverride,
        JourneyId::DeleteLocalBlueosVersion,
        JourneyId::DeleteVideoRecording,
        JourneyId::Deploy,
        JourneyId::DetectMotorDirections,
        JourneyId::DetectWifiApLoss,
        JourneyId::DisableOnboardDhcpServer,
        JourneyId::DisconnectFromWifiNetwork,
        JourneyId::DiscoverBlueosOnNetwork,
        JourneyId::DockerRegistryLogin,
        JourneyId::DownloadVideoRecording,
        JourneyId::EditExtensionDevVersion,
        JourneyId::EnableLegacyCameraSupport,
        JourneyId::EnableOnboardDhcpServer,
        JourneyId::EnablePing1dRangefinderMavlink,
        JourneyId::ForgetSavedWifiNetwork,
        JourneyId::ForceWifiNetworkPassword,
        JourneyId::FreeDiskSpace,
        JourneyId::InspectDiskUsage,
        JourneyId::InspectMavlinkMessagesInBrowser,
        JourneyId::InspectRaspberryEepromBootloader,
        JourneyId::InspectZenohNetwork,
        JourneyId::InstallCustomExtension,
        JourneyId::InstallExtension,
        JourneyId::LevelHorizon,
        JourneyId::ManageBlueosFiles,
        JourneyId::ModifyBagDatabase,
        JourneyId::MonitorInternetConnectivity,
        JourneyId::ProbeInterfaceInternetConnectivity,
        JourneyId::PullBlueosVersionWithoutSwitch,
        JourneyId::RebootOnboardComputer,
        JourneyId::ReconnectToSavedWifiNetwork,
        JourneyId::RejectInvalidWifiCredentials,
        JourneyId::RemoveCameraStream,
        JourneyId::RemoveConfiguredNmeaSocket,
        JourneyId::RemoveCustomLogo,
        JourneyId::RemoveCustomVehicleImage,
        JourneyId::RemoveSerialBridge,
        JourneyId::RenameVehicle,
        JourneyId::ResetBlueosSettings,
        JourneyId::ResetUiThemeColor,
        JourneyId::RestartAutopilot,
        JourneyId::RestoreDefaultFirmware,
        JourneyId::RunHostCommand,
        JourneyId::RunInternetSpeedTest,
        JourneyId::RunLanSpeedTest,
        JourneyId::RunMultiSizeDiskSpeedTest,
        JourneyId::RunSingleDiskSpeedTest,
        JourneyId::RunSitlSimulation,
        JourneyId::SetNetworkInterfacePriority,
        JourneyId::ShutdownOnboardComputer,
        JourneyId::StartAutopilot,
        JourneyId::StopAutopilot,
        JourneyId::SwitchLocalBlueosVersion,
        JourneyId::SyncSystemTime,
        JourneyId::ToggleHotspot,
        JourneyId::ToggleSmartHotspot,
        JourneyId::UninstallExtension,
        JourneyId::UpdateBlueosVersion,
        JourneyId::UpdateBootstrapImage,
        JourneyId::UpdateFirmwareOnline,
        JourneyId::UpdateRaspberryEepromBootloader,
        JourneyId::Upload3dModelOverride,
        JourneyId::UploadCustomFirmware,
        JourneyId::UploadCustomLogo,
        JourneyId::UploadCustomVehicleImage,
        JourneyId::VehicleFirstBoot,
        JourneyId::VerifyInternetConnectivity,
        JourneyId::ViewCameraStreams,
        JourneyId::ViewConfiguredNmeaSockets,
        JourneyId::ViewConfiguredSerialBridges,
        JourneyId::ViewDetectedSonarDevices,
        JourneyId::ViewSystemInformation,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            JourneyId::AccessBlueosWebInterface => "access_blueos_web_interface",
            JourneyId::AccessWebTerminal => "access_web_terminal",
            JourneyId::AcquireDynamicIpAddress => "acquire_dynamic_ip_address",
            JourneyId::AddCustomManifest => "add_custom_manifest",
            JourneyId::AddExternalNmeaGpsSocket => "add_external_nmea_gps_socket",
            JourneyId::ApplyParameterFile => "apply_parameter_file",
            JourneyId::AssignStaticIpAddress => "assign_static_ip_address",
            JourneyId::AutoconnectToSavedWifiNetwork => "autoconnect_to_saved_wifi_network",
            JourneyId::BrowseAvailableWebServices => "browse_available_web_services",
            JourneyId::BrowseExtensionStore => "browse_extension_store",
            JourneyId::BrowseVideoRecordings => "browse_video_recordings",
            JourneyId::CalibrateAccelerometer => "calibrate_accelerometer",
            JourneyId::CalibrateBarometer => "calibrate_barometer",
            JourneyId::CalibrateCompass => "calibrate_compass",
            JourneyId::CalibrateGyroscope => "calibrate_gyroscope",
            JourneyId::ChangeBoard => "change_board",
            JourneyId::ChangeMdnsHostname => "change_mdns_hostname",
            JourneyId::ChangeUiThemeColor => "change_ui_theme_color",
            JourneyId::ConfigureCameraStream => "configure_camera_stream",
            JourneyId::ConfigureHostDns => "configure_host_dns",
            JourneyId::ConfigureHotspotCredentials => "configure_hotspot_credentials",
            JourneyId::ConfigureInstalledExtension => "configure_installed_extension",
            JourneyId::ConfigureUvcDeviceControls => "configure_uvc_device_controls",
            JourneyId::ConfigureVideoStream => "configure_video_stream",
            JourneyId::ConnectPingViewerToSonar => "connect_ping_viewer_to_sonar",
            JourneyId::ConnectToWifiNetwork => "connect_to_wifi_network",
            JourneyId::ConnectToHiddenWifiNetwork => "connect_to_hidden_wifi_network",
            JourneyId::CreateSerialToUdpBridge => "create_serial_to_udp_bridge",
            JourneyId::Delete3dModelOverride => "delete_3d_model_override",
            JourneyId::DeleteLocalBlueosVersion => "delete_local_blueos_version",
            JourneyId::DeleteVideoRecording => "delete_video_recording",
            JourneyId::Deploy => "deploy",
            JourneyId::DetectMotorDirections => "detect_motor_directions",
            JourneyId::DetectWifiApLoss => "detect_wifi_ap_loss",
            JourneyId::DisableOnboardDhcpServer => "disable_onboard_dhcp_server",
            JourneyId::DisconnectFromWifiNetwork => "disconnect_from_wifi_network",
            JourneyId::DiscoverBlueosOnNetwork => "discover_blueos_on_network",
            JourneyId::DockerRegistryLogin => "docker_registry_login",
            JourneyId::DownloadVideoRecording => "download_video_recording",
            JourneyId::EditExtensionDevVersion => "edit_extension_dev_version",
            JourneyId::EnableLegacyCameraSupport => "enable_legacy_camera_support",
            JourneyId::EnableOnboardDhcpServer => "enable_onboard_dhcp_server",
            JourneyId::EnablePing1dRangefinderMavlink => "enable_ping1d_rangefinder_mavlink",
            JourneyId::ForgetSavedWifiNetwork => "forget_saved_wifi_network",
            JourneyId::ForceWifiNetworkPassword => "force_wifi_network_password",
            JourneyId::FreeDiskSpace => "free_disk_space",
            JourneyId::InspectDiskUsage => "inspect_disk_usage",
            JourneyId::InspectMavlinkMessagesInBrowser => "inspect_mavlink_messages_in_browser",
            JourneyId::InspectRaspberryEepromBootloader => "inspect_raspberry_eeprom_bootloader",
            JourneyId::InspectZenohNetwork => "inspect_zenoh_network",
            JourneyId::InstallCustomExtension => "install_custom_extension",
            JourneyId::InstallExtension => "install_extension",
            JourneyId::LevelHorizon => "level_horizon",
            JourneyId::ManageBlueosFiles => "manage_blueos_files",
            JourneyId::ModifyBagDatabase => "modify_bag_database",
            JourneyId::MonitorInternetConnectivity => "monitor_internet_connectivity",
            JourneyId::ProbeInterfaceInternetConnectivity => {
                "probe_interface_internet_connectivity"
            }
            JourneyId::PullBlueosVersionWithoutSwitch => "pull_blueos_version_without_switch",
            JourneyId::RebootOnboardComputer => "reboot_onboard_computer",
            JourneyId::ReconnectToSavedWifiNetwork => "reconnect_to_saved_wifi_network",
            JourneyId::RejectInvalidWifiCredentials => "reject_invalid_wifi_credentials",
            JourneyId::RemoveCameraStream => "remove_camera_stream",
            JourneyId::RemoveConfiguredNmeaSocket => "remove_configured_nmea_socket",
            JourneyId::RemoveCustomLogo => "remove_custom_logo",
            JourneyId::RemoveCustomVehicleImage => "remove_custom_vehicle_image",
            JourneyId::RemoveSerialBridge => "remove_serial_bridge",
            JourneyId::RenameVehicle => "rename_vehicle",
            JourneyId::ResetBlueosSettings => "reset_blueos_settings",
            JourneyId::ResetUiThemeColor => "reset_ui_theme_color",
            JourneyId::RestartAutopilot => "restart_autopilot",
            JourneyId::RestoreDefaultFirmware => "restore_default_firmware",
            JourneyId::RunHostCommand => "run_host_command",
            JourneyId::RunInternetSpeedTest => "run_internet_speed_test",
            JourneyId::RunLanSpeedTest => "run_lan_speed_test",
            JourneyId::RunMultiSizeDiskSpeedTest => "run_multi_size_disk_speed_test",
            JourneyId::RunSingleDiskSpeedTest => "run_single_disk_speed_test",
            JourneyId::RunSitlSimulation => "run_sitl_simulation",
            JourneyId::SetNetworkInterfacePriority => "set_network_interface_priority",
            JourneyId::ShutdownOnboardComputer => "shutdown_onboard_computer",
            JourneyId::StartAutopilot => "start_autopilot",
            JourneyId::StopAutopilot => "stop_autopilot",
            JourneyId::SwitchLocalBlueosVersion => "switch_local_blueos_version",
            JourneyId::SyncSystemTime => "sync_system_time",
            JourneyId::ToggleHotspot => "toggle_hotspot",
            JourneyId::ToggleSmartHotspot => "toggle_smart_hotspot",
            JourneyId::UninstallExtension => "uninstall_extension",
            JourneyId::UpdateBlueosVersion => "update_blueos_version",
            JourneyId::UpdateBootstrapImage => "update_bootstrap_image",
            JourneyId::UpdateFirmwareOnline => "update_firmware_online",
            JourneyId::UpdateRaspberryEepromBootloader => "update_raspberry_eeprom_bootloader",
            JourneyId::Upload3dModelOverride => "upload_3d_model_override",
            JourneyId::UploadCustomFirmware => "upload_custom_firmware",
            JourneyId::UploadCustomLogo => "upload_custom_logo",
            JourneyId::UploadCustomVehicleImage => "upload_custom_vehicle_image",
            JourneyId::VehicleFirstBoot => "vehicle_first_boot",
            JourneyId::VerifyInternetConnectivity => "verify_internet_connectivity",
            JourneyId::ViewCameraStreams => "view_camera_streams",
            JourneyId::ViewConfiguredNmeaSockets => "view_configured_nmea_sockets",
            JourneyId::ViewConfiguredSerialBridges => "view_configured_serial_bridges",
            JourneyId::ViewDetectedSonarDevices => "view_detected_sonar_devices",
            JourneyId::ViewSystemInformation => "view_system_information",
        }
    }

    pub fn from_str_id(s: &str) -> Option<JourneyId> {
        JourneyId::ALL.into_iter().find(|v| v.as_str() == s)
    }

    pub const fn rust_variant_name(self) -> &'static str {
        match self {
            JourneyId::AccessBlueosWebInterface => "AccessBlueosWebInterface",
            JourneyId::AccessWebTerminal => "AccessWebTerminal",
            JourneyId::AcquireDynamicIpAddress => "AcquireDynamicIpAddress",
            JourneyId::AddCustomManifest => "AddCustomManifest",
            JourneyId::AddExternalNmeaGpsSocket => "AddExternalNmeaGpsSocket",
            JourneyId::ApplyParameterFile => "ApplyParameterFile",
            JourneyId::AssignStaticIpAddress => "AssignStaticIpAddress",
            JourneyId::AutoconnectToSavedWifiNetwork => "AutoconnectToSavedWifiNetwork",
            JourneyId::BrowseAvailableWebServices => "BrowseAvailableWebServices",
            JourneyId::BrowseExtensionStore => "BrowseExtensionStore",
            JourneyId::BrowseVideoRecordings => "BrowseVideoRecordings",
            JourneyId::CalibrateAccelerometer => "CalibrateAccelerometer",
            JourneyId::CalibrateBarometer => "CalibrateBarometer",
            JourneyId::CalibrateCompass => "CalibrateCompass",
            JourneyId::CalibrateGyroscope => "CalibrateGyroscope",
            JourneyId::ChangeBoard => "ChangeBoard",
            JourneyId::ChangeMdnsHostname => "ChangeMdnsHostname",
            JourneyId::ChangeUiThemeColor => "ChangeUiThemeColor",
            JourneyId::ConfigureCameraStream => "ConfigureCameraStream",
            JourneyId::ConfigureHostDns => "ConfigureHostDns",
            JourneyId::ConfigureHotspotCredentials => "ConfigureHotspotCredentials",
            JourneyId::ConfigureInstalledExtension => "ConfigureInstalledExtension",
            JourneyId::ConfigureUvcDeviceControls => "ConfigureUvcDeviceControls",
            JourneyId::ConfigureVideoStream => "ConfigureVideoStream",
            JourneyId::ConnectPingViewerToSonar => "ConnectPingViewerToSonar",
            JourneyId::ConnectToWifiNetwork => "ConnectToWifiNetwork",
            JourneyId::ConnectToHiddenWifiNetwork => "ConnectToHiddenWifiNetwork",
            JourneyId::CreateSerialToUdpBridge => "CreateSerialToUdpBridge",
            JourneyId::Delete3dModelOverride => "Delete3dModelOverride",
            JourneyId::DeleteLocalBlueosVersion => "DeleteLocalBlueosVersion",
            JourneyId::DeleteVideoRecording => "DeleteVideoRecording",
            JourneyId::Deploy => "Deploy",
            JourneyId::DetectMotorDirections => "DetectMotorDirections",
            JourneyId::DetectWifiApLoss => "DetectWifiApLoss",
            JourneyId::DisableOnboardDhcpServer => "DisableOnboardDhcpServer",
            JourneyId::DisconnectFromWifiNetwork => "DisconnectFromWifiNetwork",
            JourneyId::DiscoverBlueosOnNetwork => "DiscoverBlueosOnNetwork",
            JourneyId::DockerRegistryLogin => "DockerRegistryLogin",
            JourneyId::DownloadVideoRecording => "DownloadVideoRecording",
            JourneyId::EditExtensionDevVersion => "EditExtensionDevVersion",
            JourneyId::EnableLegacyCameraSupport => "EnableLegacyCameraSupport",
            JourneyId::EnableOnboardDhcpServer => "EnableOnboardDhcpServer",
            JourneyId::EnablePing1dRangefinderMavlink => "EnablePing1dRangefinderMavlink",
            JourneyId::ForgetSavedWifiNetwork => "ForgetSavedWifiNetwork",
            JourneyId::ForceWifiNetworkPassword => "ForceWifiNetworkPassword",
            JourneyId::FreeDiskSpace => "FreeDiskSpace",
            JourneyId::InspectDiskUsage => "InspectDiskUsage",
            JourneyId::InspectMavlinkMessagesInBrowser => "InspectMavlinkMessagesInBrowser",
            JourneyId::InspectRaspberryEepromBootloader => "InspectRaspberryEepromBootloader",
            JourneyId::InspectZenohNetwork => "InspectZenohNetwork",
            JourneyId::InstallCustomExtension => "InstallCustomExtension",
            JourneyId::InstallExtension => "InstallExtension",
            JourneyId::LevelHorizon => "LevelHorizon",
            JourneyId::ManageBlueosFiles => "ManageBlueosFiles",
            JourneyId::ModifyBagDatabase => "ModifyBagDatabase",
            JourneyId::MonitorInternetConnectivity => "MonitorInternetConnectivity",
            JourneyId::ProbeInterfaceInternetConnectivity => "ProbeInterfaceInternetConnectivity",
            JourneyId::PullBlueosVersionWithoutSwitch => "PullBlueosVersionWithoutSwitch",
            JourneyId::RebootOnboardComputer => "RebootOnboardComputer",
            JourneyId::ReconnectToSavedWifiNetwork => "ReconnectToSavedWifiNetwork",
            JourneyId::RejectInvalidWifiCredentials => "RejectInvalidWifiCredentials",
            JourneyId::RemoveCameraStream => "RemoveCameraStream",
            JourneyId::RemoveConfiguredNmeaSocket => "RemoveConfiguredNmeaSocket",
            JourneyId::RemoveCustomLogo => "RemoveCustomLogo",
            JourneyId::RemoveCustomVehicleImage => "RemoveCustomVehicleImage",
            JourneyId::RemoveSerialBridge => "RemoveSerialBridge",
            JourneyId::RenameVehicle => "RenameVehicle",
            JourneyId::ResetBlueosSettings => "ResetBlueosSettings",
            JourneyId::ResetUiThemeColor => "ResetUiThemeColor",
            JourneyId::RestartAutopilot => "RestartAutopilot",
            JourneyId::RestoreDefaultFirmware => "RestoreDefaultFirmware",
            JourneyId::RunHostCommand => "RunHostCommand",
            JourneyId::RunInternetSpeedTest => "RunInternetSpeedTest",
            JourneyId::RunLanSpeedTest => "RunLanSpeedTest",
            JourneyId::RunMultiSizeDiskSpeedTest => "RunMultiSizeDiskSpeedTest",
            JourneyId::RunSingleDiskSpeedTest => "RunSingleDiskSpeedTest",
            JourneyId::RunSitlSimulation => "RunSitlSimulation",
            JourneyId::SetNetworkInterfacePriority => "SetNetworkInterfacePriority",
            JourneyId::ShutdownOnboardComputer => "ShutdownOnboardComputer",
            JourneyId::StartAutopilot => "StartAutopilot",
            JourneyId::StopAutopilot => "StopAutopilot",
            JourneyId::SwitchLocalBlueosVersion => "SwitchLocalBlueosVersion",
            JourneyId::SyncSystemTime => "SyncSystemTime",
            JourneyId::ToggleHotspot => "ToggleHotspot",
            JourneyId::ToggleSmartHotspot => "ToggleSmartHotspot",
            JourneyId::UninstallExtension => "UninstallExtension",
            JourneyId::UpdateBlueosVersion => "UpdateBlueosVersion",
            JourneyId::UpdateBootstrapImage => "UpdateBootstrapImage",
            JourneyId::UpdateFirmwareOnline => "UpdateFirmwareOnline",
            JourneyId::UpdateRaspberryEepromBootloader => "UpdateRaspberryEepromBootloader",
            JourneyId::Upload3dModelOverride => "Upload3dModelOverride",
            JourneyId::UploadCustomFirmware => "UploadCustomFirmware",
            JourneyId::UploadCustomLogo => "UploadCustomLogo",
            JourneyId::UploadCustomVehicleImage => "UploadCustomVehicleImage",
            JourneyId::VehicleFirstBoot => "VehicleFirstBoot",
            JourneyId::VerifyInternetConnectivity => "VerifyInternetConnectivity",
            JourneyId::ViewCameraStreams => "ViewCameraStreams",
            JourneyId::ViewConfiguredNmeaSockets => "ViewConfiguredNmeaSockets",
            JourneyId::ViewConfiguredSerialBridges => "ViewConfiguredSerialBridges",
            JourneyId::ViewDetectedSonarDevices => "ViewDetectedSonarDevices",
            JourneyId::ViewSystemInformation => "ViewSystemInformation",
        }
    }
    pub fn from_rust_variant_name(name: &str) -> Option<JourneyId> {
        match name {
            "AccessBlueosWebInterface" => Some(JourneyId::AccessBlueosWebInterface),
            "AccessWebTerminal" => Some(JourneyId::AccessWebTerminal),
            "AcquireDynamicIpAddress" => Some(JourneyId::AcquireDynamicIpAddress),
            "AddCustomManifest" => Some(JourneyId::AddCustomManifest),
            "AddExternalNmeaGpsSocket" => Some(JourneyId::AddExternalNmeaGpsSocket),
            "ApplyParameterFile" => Some(JourneyId::ApplyParameterFile),
            "AssignStaticIpAddress" => Some(JourneyId::AssignStaticIpAddress),
            "AutoconnectToSavedWifiNetwork" => Some(JourneyId::AutoconnectToSavedWifiNetwork),
            "BrowseAvailableWebServices" => Some(JourneyId::BrowseAvailableWebServices),
            "BrowseExtensionStore" => Some(JourneyId::BrowseExtensionStore),
            "BrowseVideoRecordings" => Some(JourneyId::BrowseVideoRecordings),
            "CalibrateAccelerometer" => Some(JourneyId::CalibrateAccelerometer),
            "CalibrateBarometer" => Some(JourneyId::CalibrateBarometer),
            "CalibrateCompass" => Some(JourneyId::CalibrateCompass),
            "CalibrateGyroscope" => Some(JourneyId::CalibrateGyroscope),
            "ChangeBoard" => Some(JourneyId::ChangeBoard),
            "ChangeMdnsHostname" => Some(JourneyId::ChangeMdnsHostname),
            "ChangeUiThemeColor" => Some(JourneyId::ChangeUiThemeColor),
            "ConfigureCameraStream" => Some(JourneyId::ConfigureCameraStream),
            "ConfigureHostDns" => Some(JourneyId::ConfigureHostDns),
            "ConfigureHotspotCredentials" => Some(JourneyId::ConfigureHotspotCredentials),
            "ConfigureInstalledExtension" => Some(JourneyId::ConfigureInstalledExtension),
            "ConfigureUvcDeviceControls" => Some(JourneyId::ConfigureUvcDeviceControls),
            "ConfigureVideoStream" => Some(JourneyId::ConfigureVideoStream),
            "ConnectPingViewerToSonar" => Some(JourneyId::ConnectPingViewerToSonar),
            "ConnectToWifiNetwork" => Some(JourneyId::ConnectToWifiNetwork),
            "ConnectToHiddenWifiNetwork" => Some(JourneyId::ConnectToHiddenWifiNetwork),
            "CreateSerialToUdpBridge" => Some(JourneyId::CreateSerialToUdpBridge),
            "Delete3dModelOverride" => Some(JourneyId::Delete3dModelOverride),
            "DeleteLocalBlueosVersion" => Some(JourneyId::DeleteLocalBlueosVersion),
            "DeleteVideoRecording" => Some(JourneyId::DeleteVideoRecording),
            "Deploy" => Some(JourneyId::Deploy),
            "DetectMotorDirections" => Some(JourneyId::DetectMotorDirections),
            "DetectWifiApLoss" => Some(JourneyId::DetectWifiApLoss),
            "DisableOnboardDhcpServer" => Some(JourneyId::DisableOnboardDhcpServer),
            "DisconnectFromWifiNetwork" => Some(JourneyId::DisconnectFromWifiNetwork),
            "DiscoverBlueosOnNetwork" => Some(JourneyId::DiscoverBlueosOnNetwork),
            "DockerRegistryLogin" => Some(JourneyId::DockerRegistryLogin),
            "DownloadVideoRecording" => Some(JourneyId::DownloadVideoRecording),
            "EditExtensionDevVersion" => Some(JourneyId::EditExtensionDevVersion),
            "EnableLegacyCameraSupport" => Some(JourneyId::EnableLegacyCameraSupport),
            "EnableOnboardDhcpServer" => Some(JourneyId::EnableOnboardDhcpServer),
            "EnablePing1dRangefinderMavlink" => Some(JourneyId::EnablePing1dRangefinderMavlink),
            "ForgetSavedWifiNetwork" => Some(JourneyId::ForgetSavedWifiNetwork),
            "ForceWifiNetworkPassword" => Some(JourneyId::ForceWifiNetworkPassword),
            "FreeDiskSpace" => Some(JourneyId::FreeDiskSpace),
            "InspectDiskUsage" => Some(JourneyId::InspectDiskUsage),
            "InspectMavlinkMessagesInBrowser" => Some(JourneyId::InspectMavlinkMessagesInBrowser),
            "InspectRaspberryEepromBootloader" => Some(JourneyId::InspectRaspberryEepromBootloader),
            "InspectZenohNetwork" => Some(JourneyId::InspectZenohNetwork),
            "InstallCustomExtension" => Some(JourneyId::InstallCustomExtension),
            "InstallExtension" => Some(JourneyId::InstallExtension),
            "LevelHorizon" => Some(JourneyId::LevelHorizon),
            "ManageBlueosFiles" => Some(JourneyId::ManageBlueosFiles),
            "ModifyBagDatabase" => Some(JourneyId::ModifyBagDatabase),
            "MonitorInternetConnectivity" => Some(JourneyId::MonitorInternetConnectivity),
            "ProbeInterfaceInternetConnectivity" => {
                Some(JourneyId::ProbeInterfaceInternetConnectivity)
            }
            "PullBlueosVersionWithoutSwitch" => Some(JourneyId::PullBlueosVersionWithoutSwitch),
            "RebootOnboardComputer" => Some(JourneyId::RebootOnboardComputer),
            "ReconnectToSavedWifiNetwork" => Some(JourneyId::ReconnectToSavedWifiNetwork),
            "RejectInvalidWifiCredentials" => Some(JourneyId::RejectInvalidWifiCredentials),
            "RemoveCameraStream" => Some(JourneyId::RemoveCameraStream),
            "RemoveConfiguredNmeaSocket" => Some(JourneyId::RemoveConfiguredNmeaSocket),
            "RemoveCustomLogo" => Some(JourneyId::RemoveCustomLogo),
            "RemoveCustomVehicleImage" => Some(JourneyId::RemoveCustomVehicleImage),
            "RemoveSerialBridge" => Some(JourneyId::RemoveSerialBridge),
            "RenameVehicle" => Some(JourneyId::RenameVehicle),
            "ResetBlueosSettings" => Some(JourneyId::ResetBlueosSettings),
            "ResetUiThemeColor" => Some(JourneyId::ResetUiThemeColor),
            "RestartAutopilot" => Some(JourneyId::RestartAutopilot),
            "RestoreDefaultFirmware" => Some(JourneyId::RestoreDefaultFirmware),
            "RunHostCommand" => Some(JourneyId::RunHostCommand),
            "RunInternetSpeedTest" => Some(JourneyId::RunInternetSpeedTest),
            "RunLanSpeedTest" => Some(JourneyId::RunLanSpeedTest),
            "RunMultiSizeDiskSpeedTest" => Some(JourneyId::RunMultiSizeDiskSpeedTest),
            "RunSingleDiskSpeedTest" => Some(JourneyId::RunSingleDiskSpeedTest),
            "RunSitlSimulation" => Some(JourneyId::RunSitlSimulation),
            "SetNetworkInterfacePriority" => Some(JourneyId::SetNetworkInterfacePriority),
            "ShutdownOnboardComputer" => Some(JourneyId::ShutdownOnboardComputer),
            "StartAutopilot" => Some(JourneyId::StartAutopilot),
            "StopAutopilot" => Some(JourneyId::StopAutopilot),
            "SwitchLocalBlueosVersion" => Some(JourneyId::SwitchLocalBlueosVersion),
            "SyncSystemTime" => Some(JourneyId::SyncSystemTime),
            "ToggleHotspot" => Some(JourneyId::ToggleHotspot),
            "ToggleSmartHotspot" => Some(JourneyId::ToggleSmartHotspot),
            "UninstallExtension" => Some(JourneyId::UninstallExtension),
            "UpdateBlueosVersion" => Some(JourneyId::UpdateBlueosVersion),
            "UpdateBootstrapImage" => Some(JourneyId::UpdateBootstrapImage),
            "UpdateFirmwareOnline" => Some(JourneyId::UpdateFirmwareOnline),
            "UpdateRaspberryEepromBootloader" => Some(JourneyId::UpdateRaspberryEepromBootloader),
            "Upload3dModelOverride" => Some(JourneyId::Upload3dModelOverride),
            "UploadCustomFirmware" => Some(JourneyId::UploadCustomFirmware),
            "UploadCustomLogo" => Some(JourneyId::UploadCustomLogo),
            "UploadCustomVehicleImage" => Some(JourneyId::UploadCustomVehicleImage),
            "VehicleFirstBoot" => Some(JourneyId::VehicleFirstBoot),
            "VerifyInternetConnectivity" => Some(JourneyId::VerifyInternetConnectivity),
            "ViewCameraStreams" => Some(JourneyId::ViewCameraStreams),
            "ViewConfiguredNmeaSockets" => Some(JourneyId::ViewConfiguredNmeaSockets),
            "ViewConfiguredSerialBridges" => Some(JourneyId::ViewConfiguredSerialBridges),
            "ViewDetectedSonarDevices" => Some(JourneyId::ViewDetectedSonarDevices),
            "ViewSystemInformation" => Some(JourneyId::ViewSystemInformation),
            _ => None,
        }
    }
}

impl fmt::Display for JourneyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Entity for JourneyId {
    const ALL: &'static [JourneyId] = &JourneyId::ALL;
    fn as_str(&self) -> &'static str {
        JourneyId::as_str(self)
    }
}
