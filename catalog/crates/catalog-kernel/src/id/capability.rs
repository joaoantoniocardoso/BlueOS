use std::fmt;

use schemars::JsonSchema;
use serde::Serialize;

use super::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, JsonSchema)]
pub enum CapabilityId {
    #[serde(rename = "access_blueos_web_interface")]
    AccessBlueosWebInterface,
    #[serde(rename = "access_mavlink_over_rest")]
    AccessMavlinkOverRest,
    #[serde(rename = "access_web_terminal")]
    AccessWebTerminal,
    #[serde(rename = "acquire_dynamic_ip")]
    AcquireDynamicIp,
    #[serde(rename = "advertise_cameras_over_mavlink")]
    AdvertiseCamerasOverMavlink,
    #[serde(rename = "advertise_mdns_domains")]
    AdvertiseMdnsDomains,
    #[serde(rename = "apply_parameter_set")]
    ApplyParameterSet,
    #[serde(rename = "assign_static_ip")]
    AssignStaticIp,
    #[serde(rename = "browse_extension_store")]
    BrowseExtensionStore,
    #[serde(rename = "browse_video_recordings")]
    BrowseVideoRecordings,
    #[serde(rename = "cache_external_http")]
    CacheExternalHttp,
    #[serde(rename = "calibrate_accelerometer")]
    CalibrateAccelerometer,
    #[serde(rename = "calibrate_barometer")]
    CalibrateBarometer,
    #[serde(rename = "calibrate_compass")]
    CalibrateCompass,
    #[serde(rename = "calibrate_gyroscope")]
    CalibrateGyroscope,
    #[serde(rename = "check_internet_connectivity")]
    CheckInternetConnectivity,
    #[serde(rename = "configure_camera_stream")]
    ConfigureCameraStream,
    #[serde(rename = "configure_extension")]
    ConfigureExtension,
    #[serde(rename = "configure_host_dns")]
    ConfigureHostDns,
    #[serde(rename = "configure_legacy_camera")]
    ConfigureLegacyCamera,
    #[serde(rename = "configure_sitl_frame")]
    ConfigureSitlFrame,
    #[serde(rename = "configure_stream_endpoints")]
    ConfigureStreamEndpoints,
    #[serde(rename = "configure_uvc_device_controls")]
    ConfigureUvcDeviceControls,
    #[serde(rename = "connect_ping_viewer_to_sonar")]
    ConnectPingViewerToSonar,
    #[serde(rename = "connect_wifi_network")]
    ConnectWifiNetwork,
    #[serde(rename = "create_nmea_socket")]
    CreateNmeaSocket,
    #[serde(rename = "create_serial_to_udp_bridge")]
    CreateSerialToUdpBridge,
    #[serde(rename = "delete_disk_paths")]
    DeleteDiskPaths,
    #[serde(rename = "delete_local_blueos_version")]
    DeleteLocalBlueosVersion,
    #[serde(rename = "delete_model_override")]
    DeleteModelOverride,
    #[serde(rename = "delete_video_recording")]
    DeleteVideoRecording,
    #[serde(rename = "deploy")]
    Deploy,
    #[serde(rename = "derive_sensor_calibration_status")]
    DeriveSensorCalibrationStatus,
    #[serde(rename = "detect_flight_controllers")]
    DetectFlightControllers,
    #[serde(rename = "detect_motor_directions")]
    DetectMotorDirections,
    #[serde(rename = "diagnose_stream_accessibility")]
    DiagnoseStreamAccessibility,
    #[serde(rename = "disable_dhcp_server")]
    DisableDhcpServer,
    #[serde(rename = "disconnect_wifi_network")]
    DisconnectWifiNetwork,
    #[serde(rename = "discover_web_services")]
    DiscoverWebServices,
    #[serde(rename = "docker_registry_login")]
    DockerRegistryLogin,
    #[serde(rename = "download_video_recording")]
    DownloadVideoRecording,
    #[serde(rename = "edit_autopilot_parameters")]
    EditAutopilotParameters,
    #[serde(rename = "edit_bag_json_store")]
    EditBagJsonStore,
    #[serde(rename = "enable_dhcp_server")]
    EnableDhcpServer,
    #[serde(rename = "enable_ping1d_mavlink_distance")]
    EnablePing1dMavlinkDistance,
    #[serde(rename = "filter_displayable_devices")]
    FilterDisplayableDevices,
    #[serde(rename = "flash_firmware")]
    FlashFirmware,
    #[serde(rename = "get_bag_value")]
    GetBagValue,
    #[serde(rename = "get_branding_logo")]
    GetBrandingLogo,
    #[serde(rename = "get_branding_vehicle_image")]
    GetBrandingVehicleImage,
    #[serde(rename = "get_current_blueos_version")]
    GetCurrentBlueosVersion,
    #[serde(rename = "get_current_bootstrap_version")]
    GetCurrentBootstrapVersion,
    #[serde(rename = "get_dhcp_server_details")]
    GetDhcpServerDetails,
    #[serde(rename = "get_dhcp_server_leases")]
    GetDhcpServerLeases,
    #[serde(rename = "get_hotspot_status")]
    GetHotspotStatus,
    #[serde(rename = "get_interface_routes")]
    GetInterfaceRoutes,
    #[serde(rename = "get_mdns_hostname")]
    GetMdnsHostname,
    #[serde(rename = "get_theme_configuration")]
    GetThemeConfiguration,
    #[serde(rename = "get_vehicle_name")]
    GetVehicleName,
    #[serde(rename = "get_wifi_status")]
    GetWifiStatus,
    #[serde(rename = "inspect_disk_usage")]
    InspectDiskUsage,
    #[serde(rename = "inspect_live_mavlink_messages")]
    InspectLiveMavlinkMessages,
    #[serde(rename = "inspect_raspberry_eeprom")]
    InspectRaspberryEeprom,
    #[serde(rename = "inspect_zenoh_network")]
    InspectZenohNetwork,
    #[serde(rename = "install_extension")]
    InstallExtension,
    #[serde(rename = "level_horizon")]
    LevelHorizon,
    #[serde(rename = "list_configured_serial_bridges")]
    ListConfiguredSerialBridges,
    #[serde(rename = "list_detected_ping_sensors")]
    ListDetectedPingSensors,
    #[serde(rename = "list_docker_accounts")]
    ListDockerAccounts,
    #[serde(rename = "list_ethernet_interfaces")]
    ListEthernetInterfaces,
    #[serde(rename = "list_local_blueos_versions")]
    ListLocalBlueosVersions,
    #[serde(rename = "list_mdns_domains")]
    ListMdnsDomains,
    #[serde(rename = "list_model_overrides")]
    ListModelOverrides,
    #[serde(rename = "list_network_interfaces")]
    ListNetworkInterfaces,
    #[serde(rename = "list_nmea_sockets")]
    ListNmeaSockets,
    #[serde(rename = "list_remote_blueos_versions")]
    ListRemoteBlueosVersions,
    #[serde(rename = "list_saved_wifi_networks")]
    ListSavedWifiNetworks,
    #[serde(rename = "manage_autopilot_lifecycle")]
    ManageAutopilotLifecycle,
    #[serde(rename = "manage_blueos_files")]
    ManageBlueosFiles,
    #[serde(rename = "manage_extension_lifecycle")]
    ManageExtensionLifecycle,
    #[serde(rename = "manage_manifests")]
    ManageManifests,
    #[serde(rename = "manage_mavlink_endpoints")]
    ManageMavlinkEndpoints,
    #[serde(rename = "manage_mavlink_router")]
    ManageMavlinkRouter,
    #[serde(rename = "manage_serial_ports")]
    ManageSerialPorts,
    #[serde(rename = "manage_thumbnail_preview")]
    ManageThumbnailPreview,
    #[serde(rename = "navigate_disk_usage")]
    NavigateDiskUsage,
    #[serde(rename = "overwrite_bag_store")]
    OverwriteBagStore,
    #[serde(rename = "probe_interface_connectivity")]
    ProbeInterfaceConnectivity,
    #[serde(rename = "provide_interactive_root_shell")]
    ProvideInteractiveRootShell,
    #[serde(rename = "provide_shell_over_websocket")]
    ProvideShellOverWebsocket,
    #[serde(rename = "provide_system_information_over_rest")]
    ProvideSystemInformationOverRest,
    #[serde(rename = "provide_webrtc_signalling")]
    ProvideWebrtcSignalling,
    #[serde(rename = "pull_blueos_version")]
    PullBlueosVersion,
    #[serde(rename = "query_vehicle_firmware_info")]
    QueryVehicleFirmwareInfo,
    #[serde(rename = "reboot_onboard_computer")]
    RebootOnboardComputer,
    #[serde(rename = "record_vehicle_data_stream")]
    RecordVehicleDataStream,
    #[serde(rename = "register_web_service")]
    RegisterWebService,
    #[serde(rename = "reload_nginx")]
    ReloadNginx,
    #[serde(rename = "remove_branding_logo")]
    RemoveBrandingLogo,
    #[serde(rename = "remove_branding_vehicle_image")]
    RemoveBrandingVehicleImage,
    #[serde(rename = "remove_camera_stream")]
    RemoveCameraStream,
    #[serde(rename = "remove_nmea_socket")]
    RemoveNmeaSocket,
    #[serde(rename = "remove_saved_wifi_network")]
    RemoveSavedWifiNetwork,
    #[serde(rename = "remove_serial_bridge")]
    RemoveSerialBridge,
    #[serde(rename = "replace_stream_configuration")]
    ReplaceStreamConfiguration,
    #[serde(rename = "report_client_ip")]
    ReportClientIp,
    #[serde(rename = "report_hardware_id")]
    ReportHardwareId,
    #[serde(rename = "report_software_id")]
    ReportSoftwareId,
    #[serde(rename = "reset_blueos_settings")]
    ResetBlueosSettings,
    #[serde(rename = "reset_theme_color")]
    ResetThemeColor,
    #[serde(rename = "retrieve_host_dns")]
    RetrieveHostDns,
    #[serde(rename = "reverse_proxy_backend_services")]
    ReverseProxyBackendServices,
    #[serde(rename = "route_pubsub_messages")]
    RoutePubsubMessages,
    #[serde(rename = "run_disk_speed_test")]
    RunDiskSpeedTest,
    #[serde(rename = "run_host_command")]
    RunHostCommand,
    #[serde(rename = "run_internet_speed_test")]
    RunInternetSpeedTest,
    #[serde(rename = "run_lan_speed_test")]
    RunLanSpeedTest,
    #[serde(rename = "run_multi_size_disk_speed_test")]
    RunMultiSizeDiskSpeedTest,
    #[serde(rename = "scan_wifi_networks")]
    ScanWifiNetworks,
    #[serde(rename = "select_flight_controller_board")]
    SelectFlightControllerBoard,
    #[serde(rename = "serve_frontend_spa")]
    ServeFrontendSpa,
    #[serde(rename = "serve_iperf_bandwidth_test")]
    ServeIperfBandwidthTest,
    #[serde(rename = "serve_webdav_uploads")]
    ServeWebdavUploads,
    #[serde(rename = "set_bag_value")]
    SetBagValue,
    #[serde(rename = "set_hotspot_credentials")]
    SetHotspotCredentials,
    #[serde(rename = "set_interface_priority")]
    SetInterfacePriority,
    #[serde(rename = "set_mdns_hostname")]
    SetMdnsHostname,
    #[serde(rename = "set_theme_color")]
    SetThemeColor,
    #[serde(rename = "set_vehicle_name")]
    SetVehicleName,
    #[serde(rename = "setup_ssh")]
    SetupSsh,
    #[serde(rename = "shutdown_onboard_computer")]
    ShutdownOnboardComputer,
    #[serde(rename = "switch_blueos_version")]
    SwitchBlueosVersion,
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
    #[serde(rename = "update_raspberry_eeprom")]
    UpdateRaspberryEeprom,
    #[serde(rename = "upload_branding_logo")]
    UploadBrandingLogo,
    #[serde(rename = "upload_branding_vehicle_image")]
    UploadBrandingVehicleImage,
    #[serde(rename = "upload_model_override")]
    UploadModelOverride,
    #[serde(rename = "view_camera_streams")]
    ViewCameraStreams,
    #[serde(rename = "view_system_information")]
    ViewSystemInformation,
}

impl CapabilityId {
    pub const ALL: [CapabilityId; 144] = [
        CapabilityId::AccessBlueosWebInterface,
        CapabilityId::AccessMavlinkOverRest,
        CapabilityId::AccessWebTerminal,
        CapabilityId::AcquireDynamicIp,
        CapabilityId::AdvertiseCamerasOverMavlink,
        CapabilityId::AdvertiseMdnsDomains,
        CapabilityId::ApplyParameterSet,
        CapabilityId::AssignStaticIp,
        CapabilityId::BrowseExtensionStore,
        CapabilityId::BrowseVideoRecordings,
        CapabilityId::CacheExternalHttp,
        CapabilityId::CalibrateAccelerometer,
        CapabilityId::CalibrateBarometer,
        CapabilityId::CalibrateCompass,
        CapabilityId::CalibrateGyroscope,
        CapabilityId::CheckInternetConnectivity,
        CapabilityId::ConfigureCameraStream,
        CapabilityId::ConfigureExtension,
        CapabilityId::ConfigureHostDns,
        CapabilityId::ConfigureLegacyCamera,
        CapabilityId::ConfigureSitlFrame,
        CapabilityId::ConfigureStreamEndpoints,
        CapabilityId::ConfigureUvcDeviceControls,
        CapabilityId::ConnectPingViewerToSonar,
        CapabilityId::ConnectWifiNetwork,
        CapabilityId::CreateNmeaSocket,
        CapabilityId::CreateSerialToUdpBridge,
        CapabilityId::DeleteDiskPaths,
        CapabilityId::DeleteLocalBlueosVersion,
        CapabilityId::DeleteModelOverride,
        CapabilityId::DeleteVideoRecording,
        CapabilityId::Deploy,
        CapabilityId::DeriveSensorCalibrationStatus,
        CapabilityId::DetectFlightControllers,
        CapabilityId::DetectMotorDirections,
        CapabilityId::DiagnoseStreamAccessibility,
        CapabilityId::DisableDhcpServer,
        CapabilityId::DisconnectWifiNetwork,
        CapabilityId::DiscoverWebServices,
        CapabilityId::DockerRegistryLogin,
        CapabilityId::DownloadVideoRecording,
        CapabilityId::EditAutopilotParameters,
        CapabilityId::EditBagJsonStore,
        CapabilityId::EnableDhcpServer,
        CapabilityId::EnablePing1dMavlinkDistance,
        CapabilityId::FilterDisplayableDevices,
        CapabilityId::FlashFirmware,
        CapabilityId::GetBagValue,
        CapabilityId::GetBrandingLogo,
        CapabilityId::GetBrandingVehicleImage,
        CapabilityId::GetCurrentBlueosVersion,
        CapabilityId::GetCurrentBootstrapVersion,
        CapabilityId::GetDhcpServerDetails,
        CapabilityId::GetDhcpServerLeases,
        CapabilityId::GetHotspotStatus,
        CapabilityId::GetInterfaceRoutes,
        CapabilityId::GetMdnsHostname,
        CapabilityId::GetThemeConfiguration,
        CapabilityId::GetVehicleName,
        CapabilityId::GetWifiStatus,
        CapabilityId::InspectDiskUsage,
        CapabilityId::InspectLiveMavlinkMessages,
        CapabilityId::InspectRaspberryEeprom,
        CapabilityId::InspectZenohNetwork,
        CapabilityId::InstallExtension,
        CapabilityId::LevelHorizon,
        CapabilityId::ListConfiguredSerialBridges,
        CapabilityId::ListDetectedPingSensors,
        CapabilityId::ListDockerAccounts,
        CapabilityId::ListEthernetInterfaces,
        CapabilityId::ListLocalBlueosVersions,
        CapabilityId::ListMdnsDomains,
        CapabilityId::ListModelOverrides,
        CapabilityId::ListNetworkInterfaces,
        CapabilityId::ListNmeaSockets,
        CapabilityId::ListRemoteBlueosVersions,
        CapabilityId::ListSavedWifiNetworks,
        CapabilityId::ManageAutopilotLifecycle,
        CapabilityId::ManageBlueosFiles,
        CapabilityId::ManageExtensionLifecycle,
        CapabilityId::ManageManifests,
        CapabilityId::ManageMavlinkEndpoints,
        CapabilityId::ManageMavlinkRouter,
        CapabilityId::ManageSerialPorts,
        CapabilityId::ManageThumbnailPreview,
        CapabilityId::NavigateDiskUsage,
        CapabilityId::OverwriteBagStore,
        CapabilityId::ProbeInterfaceConnectivity,
        CapabilityId::ProvideInteractiveRootShell,
        CapabilityId::ProvideShellOverWebsocket,
        CapabilityId::ProvideSystemInformationOverRest,
        CapabilityId::ProvideWebrtcSignalling,
        CapabilityId::PullBlueosVersion,
        CapabilityId::QueryVehicleFirmwareInfo,
        CapabilityId::RebootOnboardComputer,
        CapabilityId::RecordVehicleDataStream,
        CapabilityId::RegisterWebService,
        CapabilityId::ReloadNginx,
        CapabilityId::RemoveBrandingLogo,
        CapabilityId::RemoveBrandingVehicleImage,
        CapabilityId::RemoveCameraStream,
        CapabilityId::RemoveNmeaSocket,
        CapabilityId::RemoveSavedWifiNetwork,
        CapabilityId::RemoveSerialBridge,
        CapabilityId::ReplaceStreamConfiguration,
        CapabilityId::ReportClientIp,
        CapabilityId::ReportHardwareId,
        CapabilityId::ReportSoftwareId,
        CapabilityId::ResetBlueosSettings,
        CapabilityId::ResetThemeColor,
        CapabilityId::RetrieveHostDns,
        CapabilityId::ReverseProxyBackendServices,
        CapabilityId::RoutePubsubMessages,
        CapabilityId::RunDiskSpeedTest,
        CapabilityId::RunHostCommand,
        CapabilityId::RunInternetSpeedTest,
        CapabilityId::RunLanSpeedTest,
        CapabilityId::RunMultiSizeDiskSpeedTest,
        CapabilityId::ScanWifiNetworks,
        CapabilityId::SelectFlightControllerBoard,
        CapabilityId::ServeFrontendSpa,
        CapabilityId::ServeIperfBandwidthTest,
        CapabilityId::ServeWebdavUploads,
        CapabilityId::SetBagValue,
        CapabilityId::SetHotspotCredentials,
        CapabilityId::SetInterfacePriority,
        CapabilityId::SetMdnsHostname,
        CapabilityId::SetThemeColor,
        CapabilityId::SetVehicleName,
        CapabilityId::SetupSsh,
        CapabilityId::ShutdownOnboardComputer,
        CapabilityId::SwitchBlueosVersion,
        CapabilityId::SyncSystemTime,
        CapabilityId::ToggleHotspot,
        CapabilityId::ToggleSmartHotspot,
        CapabilityId::UninstallExtension,
        CapabilityId::UpdateBlueosVersion,
        CapabilityId::UpdateBootstrapImage,
        CapabilityId::UpdateRaspberryEeprom,
        CapabilityId::UploadBrandingLogo,
        CapabilityId::UploadBrandingVehicleImage,
        CapabilityId::UploadModelOverride,
        CapabilityId::ViewCameraStreams,
        CapabilityId::ViewSystemInformation,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            CapabilityId::AccessBlueosWebInterface => "access_blueos_web_interface",
            CapabilityId::AccessMavlinkOverRest => "access_mavlink_over_rest",
            CapabilityId::AccessWebTerminal => "access_web_terminal",
            CapabilityId::AcquireDynamicIp => "acquire_dynamic_ip",
            CapabilityId::AdvertiseCamerasOverMavlink => "advertise_cameras_over_mavlink",
            CapabilityId::AdvertiseMdnsDomains => "advertise_mdns_domains",
            CapabilityId::ApplyParameterSet => "apply_parameter_set",
            CapabilityId::AssignStaticIp => "assign_static_ip",
            CapabilityId::BrowseExtensionStore => "browse_extension_store",
            CapabilityId::BrowseVideoRecordings => "browse_video_recordings",
            CapabilityId::CacheExternalHttp => "cache_external_http",
            CapabilityId::CalibrateAccelerometer => "calibrate_accelerometer",
            CapabilityId::CalibrateBarometer => "calibrate_barometer",
            CapabilityId::CalibrateCompass => "calibrate_compass",
            CapabilityId::CalibrateGyroscope => "calibrate_gyroscope",
            CapabilityId::CheckInternetConnectivity => "check_internet_connectivity",
            CapabilityId::ConfigureCameraStream => "configure_camera_stream",
            CapabilityId::ConfigureExtension => "configure_extension",
            CapabilityId::ConfigureHostDns => "configure_host_dns",
            CapabilityId::ConfigureLegacyCamera => "configure_legacy_camera",
            CapabilityId::ConfigureSitlFrame => "configure_sitl_frame",
            CapabilityId::ConfigureStreamEndpoints => "configure_stream_endpoints",
            CapabilityId::ConfigureUvcDeviceControls => "configure_uvc_device_controls",
            CapabilityId::ConnectPingViewerToSonar => "connect_ping_viewer_to_sonar",
            CapabilityId::ConnectWifiNetwork => "connect_wifi_network",
            CapabilityId::CreateNmeaSocket => "create_nmea_socket",
            CapabilityId::CreateSerialToUdpBridge => "create_serial_to_udp_bridge",
            CapabilityId::DeleteDiskPaths => "delete_disk_paths",
            CapabilityId::DeleteLocalBlueosVersion => "delete_local_blueos_version",
            CapabilityId::DeleteModelOverride => "delete_model_override",
            CapabilityId::DeleteVideoRecording => "delete_video_recording",
            CapabilityId::Deploy => "deploy",
            CapabilityId::DeriveSensorCalibrationStatus => "derive_sensor_calibration_status",
            CapabilityId::DetectFlightControllers => "detect_flight_controllers",
            CapabilityId::DetectMotorDirections => "detect_motor_directions",
            CapabilityId::DiagnoseStreamAccessibility => "diagnose_stream_accessibility",
            CapabilityId::DisableDhcpServer => "disable_dhcp_server",
            CapabilityId::DisconnectWifiNetwork => "disconnect_wifi_network",
            CapabilityId::DiscoverWebServices => "discover_web_services",
            CapabilityId::DockerRegistryLogin => "docker_registry_login",
            CapabilityId::DownloadVideoRecording => "download_video_recording",
            CapabilityId::EditAutopilotParameters => "edit_autopilot_parameters",
            CapabilityId::EditBagJsonStore => "edit_bag_json_store",
            CapabilityId::EnableDhcpServer => "enable_dhcp_server",
            CapabilityId::EnablePing1dMavlinkDistance => "enable_ping1d_mavlink_distance",
            CapabilityId::FilterDisplayableDevices => "filter_displayable_devices",
            CapabilityId::FlashFirmware => "flash_firmware",
            CapabilityId::GetBagValue => "get_bag_value",
            CapabilityId::GetBrandingLogo => "get_branding_logo",
            CapabilityId::GetBrandingVehicleImage => "get_branding_vehicle_image",
            CapabilityId::GetCurrentBlueosVersion => "get_current_blueos_version",
            CapabilityId::GetCurrentBootstrapVersion => "get_current_bootstrap_version",
            CapabilityId::GetDhcpServerDetails => "get_dhcp_server_details",
            CapabilityId::GetDhcpServerLeases => "get_dhcp_server_leases",
            CapabilityId::GetHotspotStatus => "get_hotspot_status",
            CapabilityId::GetInterfaceRoutes => "get_interface_routes",
            CapabilityId::GetMdnsHostname => "get_mdns_hostname",
            CapabilityId::GetThemeConfiguration => "get_theme_configuration",
            CapabilityId::GetVehicleName => "get_vehicle_name",
            CapabilityId::GetWifiStatus => "get_wifi_status",
            CapabilityId::InspectDiskUsage => "inspect_disk_usage",
            CapabilityId::InspectLiveMavlinkMessages => "inspect_live_mavlink_messages",
            CapabilityId::InspectRaspberryEeprom => "inspect_raspberry_eeprom",
            CapabilityId::InspectZenohNetwork => "inspect_zenoh_network",
            CapabilityId::InstallExtension => "install_extension",
            CapabilityId::LevelHorizon => "level_horizon",
            CapabilityId::ListConfiguredSerialBridges => "list_configured_serial_bridges",
            CapabilityId::ListDetectedPingSensors => "list_detected_ping_sensors",
            CapabilityId::ListDockerAccounts => "list_docker_accounts",
            CapabilityId::ListEthernetInterfaces => "list_ethernet_interfaces",
            CapabilityId::ListLocalBlueosVersions => "list_local_blueos_versions",
            CapabilityId::ListMdnsDomains => "list_mdns_domains",
            CapabilityId::ListModelOverrides => "list_model_overrides",
            CapabilityId::ListNetworkInterfaces => "list_network_interfaces",
            CapabilityId::ListNmeaSockets => "list_nmea_sockets",
            CapabilityId::ListRemoteBlueosVersions => "list_remote_blueos_versions",
            CapabilityId::ListSavedWifiNetworks => "list_saved_wifi_networks",
            CapabilityId::ManageAutopilotLifecycle => "manage_autopilot_lifecycle",
            CapabilityId::ManageBlueosFiles => "manage_blueos_files",
            CapabilityId::ManageExtensionLifecycle => "manage_extension_lifecycle",
            CapabilityId::ManageManifests => "manage_manifests",
            CapabilityId::ManageMavlinkEndpoints => "manage_mavlink_endpoints",
            CapabilityId::ManageMavlinkRouter => "manage_mavlink_router",
            CapabilityId::ManageSerialPorts => "manage_serial_ports",
            CapabilityId::ManageThumbnailPreview => "manage_thumbnail_preview",
            CapabilityId::NavigateDiskUsage => "navigate_disk_usage",
            CapabilityId::OverwriteBagStore => "overwrite_bag_store",
            CapabilityId::ProbeInterfaceConnectivity => "probe_interface_connectivity",
            CapabilityId::ProvideInteractiveRootShell => "provide_interactive_root_shell",
            CapabilityId::ProvideShellOverWebsocket => "provide_shell_over_websocket",
            CapabilityId::ProvideSystemInformationOverRest => {
                "provide_system_information_over_rest"
            }
            CapabilityId::ProvideWebrtcSignalling => "provide_webrtc_signalling",
            CapabilityId::PullBlueosVersion => "pull_blueos_version",
            CapabilityId::QueryVehicleFirmwareInfo => "query_vehicle_firmware_info",
            CapabilityId::RebootOnboardComputer => "reboot_onboard_computer",
            CapabilityId::RecordVehicleDataStream => "record_vehicle_data_stream",
            CapabilityId::RegisterWebService => "register_web_service",
            CapabilityId::ReloadNginx => "reload_nginx",
            CapabilityId::RemoveBrandingLogo => "remove_branding_logo",
            CapabilityId::RemoveBrandingVehicleImage => "remove_branding_vehicle_image",
            CapabilityId::RemoveCameraStream => "remove_camera_stream",
            CapabilityId::RemoveNmeaSocket => "remove_nmea_socket",
            CapabilityId::RemoveSavedWifiNetwork => "remove_saved_wifi_network",
            CapabilityId::RemoveSerialBridge => "remove_serial_bridge",
            CapabilityId::ReplaceStreamConfiguration => "replace_stream_configuration",
            CapabilityId::ReportClientIp => "report_client_ip",
            CapabilityId::ReportHardwareId => "report_hardware_id",
            CapabilityId::ReportSoftwareId => "report_software_id",
            CapabilityId::ResetBlueosSettings => "reset_blueos_settings",
            CapabilityId::ResetThemeColor => "reset_theme_color",
            CapabilityId::RetrieveHostDns => "retrieve_host_dns",
            CapabilityId::ReverseProxyBackendServices => "reverse_proxy_backend_services",
            CapabilityId::RoutePubsubMessages => "route_pubsub_messages",
            CapabilityId::RunDiskSpeedTest => "run_disk_speed_test",
            CapabilityId::RunHostCommand => "run_host_command",
            CapabilityId::RunInternetSpeedTest => "run_internet_speed_test",
            CapabilityId::RunLanSpeedTest => "run_lan_speed_test",
            CapabilityId::RunMultiSizeDiskSpeedTest => "run_multi_size_disk_speed_test",
            CapabilityId::ScanWifiNetworks => "scan_wifi_networks",
            CapabilityId::SelectFlightControllerBoard => "select_flight_controller_board",
            CapabilityId::ServeFrontendSpa => "serve_frontend_spa",
            CapabilityId::ServeIperfBandwidthTest => "serve_iperf_bandwidth_test",
            CapabilityId::ServeWebdavUploads => "serve_webdav_uploads",
            CapabilityId::SetBagValue => "set_bag_value",
            CapabilityId::SetHotspotCredentials => "set_hotspot_credentials",
            CapabilityId::SetInterfacePriority => "set_interface_priority",
            CapabilityId::SetMdnsHostname => "set_mdns_hostname",
            CapabilityId::SetThemeColor => "set_theme_color",
            CapabilityId::SetVehicleName => "set_vehicle_name",
            CapabilityId::SetupSsh => "setup_ssh",
            CapabilityId::ShutdownOnboardComputer => "shutdown_onboard_computer",
            CapabilityId::SwitchBlueosVersion => "switch_blueos_version",
            CapabilityId::SyncSystemTime => "sync_system_time",
            CapabilityId::ToggleHotspot => "toggle_hotspot",
            CapabilityId::ToggleSmartHotspot => "toggle_smart_hotspot",
            CapabilityId::UninstallExtension => "uninstall_extension",
            CapabilityId::UpdateBlueosVersion => "update_blueos_version",
            CapabilityId::UpdateBootstrapImage => "update_bootstrap_image",
            CapabilityId::UpdateRaspberryEeprom => "update_raspberry_eeprom",
            CapabilityId::UploadBrandingLogo => "upload_branding_logo",
            CapabilityId::UploadBrandingVehicleImage => "upload_branding_vehicle_image",
            CapabilityId::UploadModelOverride => "upload_model_override",
            CapabilityId::ViewCameraStreams => "view_camera_streams",
            CapabilityId::ViewSystemInformation => "view_system_information",
        }
    }

    pub fn from_str_id(s: &str) -> Option<CapabilityId> {
        CapabilityId::ALL.into_iter().find(|v| v.as_str() == s)
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Entity for CapabilityId {
    const ALL: &'static [CapabilityId] = &CapabilityId::ALL;
    fn as_str(&self) -> &'static str {
        CapabilityId::as_str(self)
    }
}
