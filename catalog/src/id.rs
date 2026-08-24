use std::fmt;

use schemars::JsonSchema;
use serde::Serialize;

pub trait Entity: Copy + Sized + 'static {
    const ALL: &'static [Self];
    fn as_str(&self) -> &'static str;
}

/// Identity of a cataloged BlueOS service. Closed set of the 26 processes launched by
/// `core/start-blueos-core`. Each variant serializes to its canonical id string (explicit
/// `rename` on every variant so the JSON is exact and independent of `rename_all` heuristics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, JsonSchema)]
pub enum ServiceId {
    #[serde(rename = "ardupilot_manager")]
    ArdupilotManager,
    #[serde(rename = "bag_of_holding")]
    BagOfHolding,
    #[serde(rename = "beacon")]
    Beacon,
    #[serde(rename = "bridget")]
    Bridget,
    #[serde(rename = "cable_guy")]
    CableGuy,
    #[serde(rename = "commander")]
    Commander,
    #[serde(rename = "customization")]
    Customization,
    #[serde(rename = "disk_usage")]
    DiskUsage,
    #[serde(rename = "filebrowser")]
    Filebrowser,
    #[serde(rename = "helper")]
    Helper,
    #[serde(rename = "iperf3")]
    Iperf3,
    #[serde(rename = "kraken")]
    Kraken,
    #[serde(rename = "linux2rest")]
    Linux2rest,
    #[serde(rename = "mavlink2rest")]
    Mavlink2rest,
    #[serde(rename = "mavlink-camera-manager")]
    MavlinkCameraManager,
    #[serde(rename = "nginx")]
    Nginx,
    #[serde(rename = "nmea_injector")]
    NmeaInjector,
    #[serde(rename = "pardal")]
    Pardal,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "recorder")]
    Recorder,
    #[serde(rename = "recorder_extractor")]
    RecorderExtractor,
    #[serde(rename = "ttyd")]
    Ttyd,
    #[serde(rename = "user_terminal")]
    UserTerminal,
    #[serde(rename = "versionchooser")]
    Versionchooser,
    #[serde(rename = "wifi")]
    Wifi,
    #[serde(rename = "zenohd")]
    Zenohd,
}

impl ServiceId {
    /// Every cataloged service, in declaration order.
    pub const ALL: [ServiceId; 26] = [
        ServiceId::ArdupilotManager,
        ServiceId::BagOfHolding,
        ServiceId::Beacon,
        ServiceId::Bridget,
        ServiceId::CableGuy,
        ServiceId::Commander,
        ServiceId::Customization,
        ServiceId::DiskUsage,
        ServiceId::Filebrowser,
        ServiceId::Helper,
        ServiceId::Iperf3,
        ServiceId::Kraken,
        ServiceId::Linux2rest,
        ServiceId::Mavlink2rest,
        ServiceId::MavlinkCameraManager,
        ServiceId::Nginx,
        ServiceId::NmeaInjector,
        ServiceId::Pardal,
        ServiceId::Ping,
        ServiceId::Recorder,
        ServiceId::RecorderExtractor,
        ServiceId::Ttyd,
        ServiceId::UserTerminal,
        ServiceId::Versionchooser,
        ServiceId::Wifi,
        ServiceId::Zenohd,
    ];

    /// The canonical id string (matches the serialized form).
    pub const fn as_str(&self) -> &'static str {
        match self {
            ServiceId::ArdupilotManager => "ardupilot_manager",
            ServiceId::BagOfHolding => "bag_of_holding",
            ServiceId::Beacon => "beacon",
            ServiceId::Bridget => "bridget",
            ServiceId::CableGuy => "cable_guy",
            ServiceId::Commander => "commander",
            ServiceId::Customization => "customization",
            ServiceId::DiskUsage => "disk_usage",
            ServiceId::Filebrowser => "filebrowser",
            ServiceId::Helper => "helper",
            ServiceId::Iperf3 => "iperf3",
            ServiceId::Kraken => "kraken",
            ServiceId::Linux2rest => "linux2rest",
            ServiceId::Mavlink2rest => "mavlink2rest",
            ServiceId::MavlinkCameraManager => "mavlink-camera-manager",
            ServiceId::Nginx => "nginx",
            ServiceId::NmeaInjector => "nmea_injector",
            ServiceId::Pardal => "pardal",
            ServiceId::Ping => "ping",
            ServiceId::Recorder => "recorder",
            ServiceId::RecorderExtractor => "recorder_extractor",
            ServiceId::Ttyd => "ttyd",
            ServiceId::UserTerminal => "user_terminal",
            ServiceId::Versionchooser => "versionchooser",
            ServiceId::Wifi => "wifi",
            ServiceId::Zenohd => "zenohd",
        }
    }

    /// Resolve a `start-blueos-core` process/tmux name to its canonical service, mapping the two
    /// known tmux aliases (`autopilot` -> ardupilot_manager, `video` -> mavlink-camera-manager).
    pub fn from_process_name(name: &str) -> Option<ServiceId> {
        match name {
            "autopilot" => Some(ServiceId::ArdupilotManager),
            "video" => Some(ServiceId::MavlinkCameraManager),
            other => ServiceId::ALL.into_iter().find(|id| id.as_str() == other),
        }
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Entity for ServiceId {
    const ALL: &'static [ServiceId] = &ServiceId::ALL;
    fn as_str(&self) -> &'static str {
        ServiceId::as_str(self)
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct TcpPort(pub u16);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct PathRef(pub &'static str);

// Externally tagged: internal tagging (`tag = "kind"`) cannot serialize a newtype
// variant wrapping a primitive (`Literal(u16)`). Serializes as {"literal": 8000} / {"env": "VAR"}.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PortRef {
    Literal(u16),
    Env(&'static str),
}
