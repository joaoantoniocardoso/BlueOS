use std::fmt;

use schemars::JsonSchema;
use serde::Serialize;

use crate::id::{CapabilityId, ServiceId};
use crate::page::PageId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
pub enum Aggregate {
    #[serde(rename = "autopilot")]
    Autopilot,
    #[serde(rename = "branding_ui")]
    BrandingUi,
    #[serde(rename = "camera")]
    Camera,
    #[serde(rename = "extensions")]
    Extensions,
    #[serde(rename = "files_kv")]
    FilesKv,
    #[serde(rename = "gps_nmea")]
    GpsNmea,
    #[serde(rename = "host_control")]
    HostControl,
    #[serde(rename = "identity_discovery")]
    IdentityDiscovery,
    #[serde(rename = "mavlink")]
    Mavlink,
    #[serde(rename = "net_diagnostics")]
    NetDiagnostics,
    #[serde(rename = "platform_infra")]
    PlatformInfra,
    #[serde(rename = "recording")]
    Recording,
    #[serde(rename = "serial_bridge")]
    SerialBridge,
    #[serde(rename = "shell_access")]
    ShellAccess,
    #[serde(rename = "sonar")]
    Sonar,
    #[serde(rename = "storage")]
    Storage,
    #[serde(rename = "versioning")]
    Versioning,
    #[serde(rename = "wired_network")]
    WiredNetwork,
    #[serde(rename = "wireless_network")]
    WirelessNetwork,
}

impl Aggregate {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Aggregate::Autopilot => "autopilot",
            Aggregate::BrandingUi => "branding_ui",
            Aggregate::Camera => "camera",
            Aggregate::Extensions => "extensions",
            Aggregate::FilesKv => "files_kv",
            Aggregate::GpsNmea => "gps_nmea",
            Aggregate::HostControl => "host_control",
            Aggregate::IdentityDiscovery => "identity_discovery",
            Aggregate::Mavlink => "mavlink",
            Aggregate::NetDiagnostics => "net_diagnostics",
            Aggregate::PlatformInfra => "platform_infra",
            Aggregate::Recording => "recording",
            Aggregate::SerialBridge => "serial_bridge",
            Aggregate::ShellAccess => "shell_access",
            Aggregate::Sonar => "sonar",
            Aggregate::Storage => "storage",
            Aggregate::Versioning => "versioning",
            Aggregate::WiredNetwork => "wired_network",
            Aggregate::WirelessNetwork => "wireless_network",
        }
    }
}

impl fmt::Display for Aggregate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct CapabilityDef {
    pub id: CapabilityId,
    pub aggregate: Aggregate,
    pub owner: ServiceId,
}

pub const CAPABILITIES: &[CapabilityDef] = &[
    CapabilityDef {
        id: CapabilityId::ManageAutopilotLifecycle,
        aggregate: Aggregate::Autopilot,
        owner: ServiceId::ArdupilotManager,
    },
    CapabilityDef {
        id: CapabilityId::ManageMavlinkEndpoints,
        aggregate: Aggregate::Mavlink,
        owner: ServiceId::ArdupilotManager,
    },
    CapabilityDef {
        id: CapabilityId::ManageMavlinkRouter,
        aggregate: Aggregate::Mavlink,
        owner: ServiceId::ArdupilotManager,
    },
    CapabilityDef {
        id: CapabilityId::FlashFirmware,
        aggregate: Aggregate::Autopilot,
        owner: ServiceId::ArdupilotManager,
    },
    CapabilityDef {
        id: CapabilityId::SelectFlightControllerBoard,
        aggregate: Aggregate::Autopilot,
        owner: ServiceId::ArdupilotManager,
    },
    CapabilityDef {
        id: CapabilityId::DetectFlightControllers,
        aggregate: Aggregate::Autopilot,
        owner: ServiceId::ArdupilotManager,
    },
    CapabilityDef {
        id: CapabilityId::ManageSerialPorts,
        aggregate: Aggregate::SerialBridge,
        owner: ServiceId::ArdupilotManager,
    },
    CapabilityDef {
        id: CapabilityId::QueryVehicleFirmwareInfo,
        aggregate: Aggregate::Autopilot,
        owner: ServiceId::ArdupilotManager,
    },
    CapabilityDef {
        id: CapabilityId::ConfigureSitlFrame,
        aggregate: Aggregate::Autopilot,
        owner: ServiceId::ArdupilotManager,
    },
    CapabilityDef {
        id: CapabilityId::EditBagJsonStore,
        aggregate: Aggregate::FilesKv,
        owner: ServiceId::BagOfHolding,
    },
    CapabilityDef {
        id: CapabilityId::SetBagValue,
        aggregate: Aggregate::FilesKv,
        owner: ServiceId::BagOfHolding,
    },
    CapabilityDef {
        id: CapabilityId::GetBagValue,
        aggregate: Aggregate::FilesKv,
        owner: ServiceId::BagOfHolding,
    },
    CapabilityDef {
        id: CapabilityId::OverwriteBagStore,
        aggregate: Aggregate::FilesKv,
        owner: ServiceId::BagOfHolding,
    },
    CapabilityDef {
        id: CapabilityId::SetVehicleName,
        aggregate: Aggregate::IdentityDiscovery,
        owner: ServiceId::Beacon,
    },
    CapabilityDef {
        id: CapabilityId::SetMdnsHostname,
        aggregate: Aggregate::IdentityDiscovery,
        owner: ServiceId::Beacon,
    },
    CapabilityDef {
        id: CapabilityId::AdvertiseMdnsDomains,
        aggregate: Aggregate::IdentityDiscovery,
        owner: ServiceId::Beacon,
    },
    CapabilityDef {
        id: CapabilityId::GetVehicleName,
        aggregate: Aggregate::IdentityDiscovery,
        owner: ServiceId::Beacon,
    },
    CapabilityDef {
        id: CapabilityId::GetMdnsHostname,
        aggregate: Aggregate::IdentityDiscovery,
        owner: ServiceId::Beacon,
    },
    CapabilityDef {
        id: CapabilityId::ListMdnsDomains,
        aggregate: Aggregate::IdentityDiscovery,
        owner: ServiceId::Beacon,
    },
    CapabilityDef {
        id: CapabilityId::ReportClientIp,
        aggregate: Aggregate::NetDiagnostics,
        owner: ServiceId::Beacon,
    },
    CapabilityDef {
        id: CapabilityId::ListConfiguredSerialBridges,
        aggregate: Aggregate::SerialBridge,
        owner: ServiceId::Bridget,
    },
    CapabilityDef {
        id: CapabilityId::CreateSerialToUdpBridge,
        aggregate: Aggregate::SerialBridge,
        owner: ServiceId::Bridget,
    },
    CapabilityDef {
        id: CapabilityId::RemoveSerialBridge,
        aggregate: Aggregate::SerialBridge,
        owner: ServiceId::Bridget,
    },
    CapabilityDef {
        id: CapabilityId::AssignStaticIp,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::AcquireDynamicIp,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::EnableDhcpServer,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::DisableDhcpServer,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::SetInterfacePriority,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::ConfigureHostDns,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::ListNetworkInterfaces,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::ListEthernetInterfaces,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::RetrieveHostDns,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::GetInterfaceRoutes,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::GetDhcpServerDetails,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::GetDhcpServerLeases,
        aggregate: Aggregate::WiredNetwork,
        owner: ServiceId::CableGuy,
    },
    CapabilityDef {
        id: CapabilityId::RebootOnboardComputer,
        aggregate: Aggregate::HostControl,
        owner: ServiceId::Commander,
    },
    CapabilityDef {
        id: CapabilityId::ShutdownOnboardComputer,
        aggregate: Aggregate::HostControl,
        owner: ServiceId::Commander,
    },
    CapabilityDef {
        id: CapabilityId::SyncSystemTime,
        aggregate: Aggregate::HostControl,
        owner: ServiceId::Commander,
    },
    CapabilityDef {
        id: CapabilityId::ConfigureLegacyCamera,
        aggregate: Aggregate::Camera,
        owner: ServiceId::Commander,
    },
    CapabilityDef {
        id: CapabilityId::InspectRaspberryEeprom,
        aggregate: Aggregate::HostControl,
        owner: ServiceId::Commander,
    },
    CapabilityDef {
        id: CapabilityId::UpdateRaspberryEeprom,
        aggregate: Aggregate::HostControl,
        owner: ServiceId::Commander,
    },
    CapabilityDef {
        id: CapabilityId::ResetBlueosSettings,
        aggregate: Aggregate::Versioning,
        owner: ServiceId::Commander,
    },
    CapabilityDef {
        id: CapabilityId::RunHostCommand,
        aggregate: Aggregate::HostControl,
        owner: ServiceId::Commander,
    },
    CapabilityDef {
        id: CapabilityId::SetupSsh,
        aggregate: Aggregate::HostControl,
        owner: ServiceId::Commander,
    },
    CapabilityDef {
        id: CapabilityId::SetThemeColor,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::ResetThemeColor,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::GetThemeConfiguration,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::UploadBrandingLogo,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::RemoveBrandingLogo,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::GetBrandingLogo,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::UploadBrandingVehicleImage,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::RemoveBrandingVehicleImage,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::GetBrandingVehicleImage,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::UploadModelOverride,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::DeleteModelOverride,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::ListModelOverrides,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Customization,
    },
    CapabilityDef {
        id: CapabilityId::InspectDiskUsage,
        aggregate: Aggregate::Storage,
        owner: ServiceId::DiskUsage,
    },
    CapabilityDef {
        id: CapabilityId::NavigateDiskUsage,
        aggregate: Aggregate::Storage,
        owner: ServiceId::DiskUsage,
    },
    CapabilityDef {
        id: CapabilityId::DeleteDiskPaths,
        aggregate: Aggregate::Storage,
        owner: ServiceId::DiskUsage,
    },
    CapabilityDef {
        id: CapabilityId::RunDiskSpeedTest,
        aggregate: Aggregate::Storage,
        owner: ServiceId::DiskUsage,
    },
    CapabilityDef {
        id: CapabilityId::RunMultiSizeDiskSpeedTest,
        aggregate: Aggregate::Storage,
        owner: ServiceId::DiskUsage,
    },
    CapabilityDef {
        id: CapabilityId::ManageBlueosFiles,
        aggregate: Aggregate::FilesKv,
        owner: ServiceId::Filebrowser,
    },
    CapabilityDef {
        id: CapabilityId::CheckInternetConnectivity,
        aggregate: Aggregate::NetDiagnostics,
        owner: ServiceId::Helper,
    },
    CapabilityDef {
        id: CapabilityId::DiscoverWebServices,
        aggregate: Aggregate::IdentityDiscovery,
        owner: ServiceId::Helper,
    },
    CapabilityDef {
        id: CapabilityId::ProbeInterfaceConnectivity,
        aggregate: Aggregate::NetDiagnostics,
        owner: ServiceId::Helper,
    },
    CapabilityDef {
        id: CapabilityId::ReportHardwareId,
        aggregate: Aggregate::IdentityDiscovery,
        owner: ServiceId::Helper,
    },
    CapabilityDef {
        id: CapabilityId::ReportSoftwareId,
        aggregate: Aggregate::IdentityDiscovery,
        owner: ServiceId::Helper,
    },
    CapabilityDef {
        id: CapabilityId::RegisterWebService,
        aggregate: Aggregate::IdentityDiscovery,
        owner: ServiceId::Helper,
    },
    CapabilityDef {
        id: CapabilityId::ReloadNginx,
        aggregate: Aggregate::PlatformInfra,
        owner: ServiceId::Helper,
    },
    CapabilityDef {
        id: CapabilityId::ServeIperfBandwidthTest,
        aggregate: Aggregate::NetDiagnostics,
        owner: ServiceId::Iperf3,
    },
    CapabilityDef {
        id: CapabilityId::BrowseExtensionStore,
        aggregate: Aggregate::Extensions,
        owner: ServiceId::Kraken,
    },
    CapabilityDef {
        id: CapabilityId::ConfigureExtension,
        aggregate: Aggregate::Extensions,
        owner: ServiceId::Kraken,
    },
    CapabilityDef {
        id: CapabilityId::InstallExtension,
        aggregate: Aggregate::Extensions,
        owner: ServiceId::Kraken,
    },
    CapabilityDef {
        id: CapabilityId::ManageExtensionLifecycle,
        aggregate: Aggregate::Extensions,
        owner: ServiceId::Kraken,
    },
    CapabilityDef {
        id: CapabilityId::ManageManifests,
        aggregate: Aggregate::Extensions,
        owner: ServiceId::Kraken,
    },
    CapabilityDef {
        id: CapabilityId::UninstallExtension,
        aggregate: Aggregate::Extensions,
        owner: ServiceId::Kraken,
    },
    CapabilityDef {
        id: CapabilityId::ViewSystemInformation,
        aggregate: Aggregate::HostControl,
        owner: ServiceId::Linux2rest,
    },
    CapabilityDef {
        id: CapabilityId::ProvideSystemInformationOverRest,
        aggregate: Aggregate::HostControl,
        owner: ServiceId::Linux2rest,
    },
    CapabilityDef {
        id: CapabilityId::ViewCameraStreams,
        aggregate: Aggregate::Camera,
        owner: ServiceId::MavlinkCameraManager,
    },
    CapabilityDef {
        id: CapabilityId::ConfigureCameraStream,
        aggregate: Aggregate::Camera,
        owner: ServiceId::MavlinkCameraManager,
    },
    CapabilityDef {
        id: CapabilityId::RemoveCameraStream,
        aggregate: Aggregate::Camera,
        owner: ServiceId::MavlinkCameraManager,
    },
    CapabilityDef {
        id: CapabilityId::ConfigureUvcDeviceControls,
        aggregate: Aggregate::Camera,
        owner: ServiceId::MavlinkCameraManager,
    },
    CapabilityDef {
        id: CapabilityId::ProvideWebrtcSignalling,
        aggregate: Aggregate::Camera,
        owner: ServiceId::MavlinkCameraManager,
    },
    CapabilityDef {
        id: CapabilityId::AdvertiseCamerasOverMavlink,
        aggregate: Aggregate::Mavlink,
        owner: ServiceId::MavlinkCameraManager,
    },
    CapabilityDef {
        id: CapabilityId::InspectLiveMavlinkMessages,
        aggregate: Aggregate::Mavlink,
        owner: ServiceId::Mavlink2rest,
    },
    CapabilityDef {
        id: CapabilityId::AccessMavlinkOverRest,
        aggregate: Aggregate::Mavlink,
        owner: ServiceId::Mavlink2rest,
    },
    CapabilityDef {
        id: CapabilityId::ListNmeaSockets,
        aggregate: Aggregate::GpsNmea,
        owner: ServiceId::NmeaInjector,
    },
    CapabilityDef {
        id: CapabilityId::CreateNmeaSocket,
        aggregate: Aggregate::GpsNmea,
        owner: ServiceId::NmeaInjector,
    },
    CapabilityDef {
        id: CapabilityId::RemoveNmeaSocket,
        aggregate: Aggregate::GpsNmea,
        owner: ServiceId::NmeaInjector,
    },
    CapabilityDef {
        id: CapabilityId::AccessBlueosWebInterface,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Nginx,
    },
    CapabilityDef {
        id: CapabilityId::ServeFrontendSpa,
        aggregate: Aggregate::BrandingUi,
        owner: ServiceId::Nginx,
    },
    CapabilityDef {
        id: CapabilityId::ReverseProxyBackendServices,
        aggregate: Aggregate::PlatformInfra,
        owner: ServiceId::Nginx,
    },
    CapabilityDef {
        id: CapabilityId::ServeWebdavUploads,
        aggregate: Aggregate::FilesKv,
        owner: ServiceId::Nginx,
    },
    CapabilityDef {
        id: CapabilityId::CacheExternalHttp,
        aggregate: Aggregate::PlatformInfra,
        owner: ServiceId::Nginx,
    },
    CapabilityDef {
        id: CapabilityId::RunLanSpeedTest,
        aggregate: Aggregate::NetDiagnostics,
        owner: ServiceId::Pardal,
    },
    CapabilityDef {
        id: CapabilityId::RunInternetSpeedTest,
        aggregate: Aggregate::NetDiagnostics,
        owner: ServiceId::Pardal,
    },
    CapabilityDef {
        id: CapabilityId::ListDetectedPingSensors,
        aggregate: Aggregate::Sonar,
        owner: ServiceId::Ping,
    },
    CapabilityDef {
        id: CapabilityId::ConnectPingViewerToSonar,
        aggregate: Aggregate::Sonar,
        owner: ServiceId::Ping,
    },
    CapabilityDef {
        id: CapabilityId::EnablePing1dMavlinkDistance,
        aggregate: Aggregate::Sonar,
        owner: ServiceId::Ping,
    },
    CapabilityDef {
        id: CapabilityId::RecordVehicleDataStream,
        aggregate: Aggregate::Recording,
        owner: ServiceId::Recorder,
    },
    CapabilityDef {
        id: CapabilityId::BrowseVideoRecordings,
        aggregate: Aggregate::Recording,
        owner: ServiceId::RecorderExtractor,
    },
    CapabilityDef {
        id: CapabilityId::DownloadVideoRecording,
        aggregate: Aggregate::Recording,
        owner: ServiceId::RecorderExtractor,
    },
    CapabilityDef {
        id: CapabilityId::DeleteVideoRecording,
        aggregate: Aggregate::Recording,
        owner: ServiceId::RecorderExtractor,
    },
    CapabilityDef {
        id: CapabilityId::AccessWebTerminal,
        aggregate: Aggregate::ShellAccess,
        owner: ServiceId::Ttyd,
    },
    CapabilityDef {
        id: CapabilityId::ProvideShellOverWebsocket,
        aggregate: Aggregate::ShellAccess,
        owner: ServiceId::Ttyd,
    },
    CapabilityDef {
        id: CapabilityId::ProvideInteractiveRootShell,
        aggregate: Aggregate::ShellAccess,
        owner: ServiceId::UserTerminal,
    },
    CapabilityDef {
        id: CapabilityId::UpdateBlueosVersion,
        aggregate: Aggregate::Versioning,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::SwitchBlueosVersion,
        aggregate: Aggregate::Versioning,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::PullBlueosVersion,
        aggregate: Aggregate::Versioning,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::DeleteLocalBlueosVersion,
        aggregate: Aggregate::Versioning,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::DockerRegistryLogin,
        aggregate: Aggregate::Extensions,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::UpdateBootstrapImage,
        aggregate: Aggregate::Versioning,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::ListLocalBlueosVersions,
        aggregate: Aggregate::Versioning,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::ListRemoteBlueosVersions,
        aggregate: Aggregate::Versioning,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::GetCurrentBlueosVersion,
        aggregate: Aggregate::Versioning,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::GetCurrentBootstrapVersion,
        aggregate: Aggregate::Versioning,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::ListDockerAccounts,
        aggregate: Aggregate::Extensions,
        owner: ServiceId::Versionchooser,
    },
    CapabilityDef {
        id: CapabilityId::ConnectWifiNetwork,
        aggregate: Aggregate::WirelessNetwork,
        owner: ServiceId::Wifi,
    },
    CapabilityDef {
        id: CapabilityId::DisconnectWifiNetwork,
        aggregate: Aggregate::WirelessNetwork,
        owner: ServiceId::Wifi,
    },
    CapabilityDef {
        id: CapabilityId::RemoveSavedWifiNetwork,
        aggregate: Aggregate::WirelessNetwork,
        owner: ServiceId::Wifi,
    },
    CapabilityDef {
        id: CapabilityId::ToggleHotspot,
        aggregate: Aggregate::WirelessNetwork,
        owner: ServiceId::Wifi,
    },
    CapabilityDef {
        id: CapabilityId::SetHotspotCredentials,
        aggregate: Aggregate::WirelessNetwork,
        owner: ServiceId::Wifi,
    },
    CapabilityDef {
        id: CapabilityId::ToggleSmartHotspot,
        aggregate: Aggregate::WirelessNetwork,
        owner: ServiceId::Wifi,
    },
    CapabilityDef {
        id: CapabilityId::ScanWifiNetworks,
        aggregate: Aggregate::WirelessNetwork,
        owner: ServiceId::Wifi,
    },
    CapabilityDef {
        id: CapabilityId::GetWifiStatus,
        aggregate: Aggregate::WirelessNetwork,
        owner: ServiceId::Wifi,
    },
    CapabilityDef {
        id: CapabilityId::ListSavedWifiNetworks,
        aggregate: Aggregate::WirelessNetwork,
        owner: ServiceId::Wifi,
    },
    CapabilityDef {
        id: CapabilityId::GetHotspotStatus,
        aggregate: Aggregate::WirelessNetwork,
        owner: ServiceId::Wifi,
    },
    CapabilityDef {
        id: CapabilityId::InspectZenohNetwork,
        aggregate: Aggregate::PlatformInfra,
        owner: ServiceId::Zenohd,
    },
    CapabilityDef {
        id: CapabilityId::RoutePubsubMessages,
        aggregate: Aggregate::PlatformInfra,
        owner: ServiceId::Zenohd,
    },
];

pub fn capability_def(id: CapabilityId) -> Option<&'static CapabilityDef> {
    CAPABILITIES.iter().find(|def| def.id == id)
}

pub struct FrontendCapabilityDef {
    pub id: CapabilityId,
    pub aggregate: Aggregate,
    pub owner: PageId,
}

pub const FRONTEND_CAPABILITIES: &[FrontendCapabilityDef] = &[
    FrontendCapabilityDef {
        id: CapabilityId::CalibrateAccelerometer,
        aggregate: Aggregate::Autopilot,
        owner: PageId::VehicleSetup,
    },
    FrontendCapabilityDef {
        id: CapabilityId::CalibrateBarometer,
        aggregate: Aggregate::Autopilot,
        owner: PageId::VehicleSetup,
    },
    FrontendCapabilityDef {
        id: CapabilityId::CalibrateCompass,
        aggregate: Aggregate::Autopilot,
        owner: PageId::VehicleSetup,
    },
    FrontendCapabilityDef {
        id: CapabilityId::CalibrateGyroscope,
        aggregate: Aggregate::Autopilot,
        owner: PageId::VehicleSetup,
    },
    FrontendCapabilityDef {
        id: CapabilityId::DeriveSensorCalibrationStatus,
        aggregate: Aggregate::Autopilot,
        owner: PageId::VehicleSetup,
    },
    FrontendCapabilityDef {
        id: CapabilityId::DetectMotorDirections,
        aggregate: Aggregate::Autopilot,
        owner: PageId::VehicleSetup,
    },
    FrontendCapabilityDef {
        id: CapabilityId::LevelHorizon,
        aggregate: Aggregate::Autopilot,
        owner: PageId::VehicleSetup,
    },
    FrontendCapabilityDef {
        id: CapabilityId::EditAutopilotParameters,
        aggregate: Aggregate::Autopilot,
        owner: PageId::ParameterEditor,
    },
    FrontendCapabilityDef {
        id: CapabilityId::ApplyParameterSet,
        aggregate: Aggregate::Autopilot,
        owner: PageId::ParameterEditor,
    },
    FrontendCapabilityDef {
        id: CapabilityId::ConfigureStreamEndpoints,
        aggregate: Aggregate::Camera,
        owner: PageId::VideoManager,
    },
    FrontendCapabilityDef {
        id: CapabilityId::DiagnoseStreamAccessibility,
        aggregate: Aggregate::Camera,
        owner: PageId::VideoManager,
    },
    FrontendCapabilityDef {
        id: CapabilityId::FilterDisplayableDevices,
        aggregate: Aggregate::Camera,
        owner: PageId::VideoManager,
    },
    FrontendCapabilityDef {
        id: CapabilityId::ManageThumbnailPreview,
        aggregate: Aggregate::Camera,
        owner: PageId::VideoManager,
    },
    FrontendCapabilityDef {
        id: CapabilityId::ReplaceStreamConfiguration,
        aggregate: Aggregate::Camera,
        owner: PageId::VideoManager,
    },
];

pub fn frontend_capability_def(id: CapabilityId) -> Option<&'static FrontendCapabilityDef> {
    FRONTEND_CAPABILITIES.iter().find(|def| def.id == id)
}
