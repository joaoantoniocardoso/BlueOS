use std::fmt;

use schemars::JsonSchema;
use serde::Serialize;

use super::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, JsonSchema)]
pub enum PageId {
    #[serde(rename = "main")]
    Main,
    #[serde(rename = "autopilot")]
    Autopilot,
    #[serde(rename = "vehicle_setup")]
    VehicleSetup,
    #[serde(rename = "pings")]
    Pings,
    #[serde(rename = "log_browser")]
    LogBrowser,
    #[serde(rename = "endpoints")]
    Endpoints,
    #[serde(rename = "file_browser")]
    FileBrowser,
    #[serde(rename = "disk")]
    Disk,
    #[serde(rename = "terminal")]
    Terminal,
    #[serde(rename = "version_chooser")]
    VersionChooser,
    #[serde(rename = "video_manager")]
    VideoManager,
    #[serde(rename = "records")]
    Records,
    #[serde(rename = "bridges")]
    Bridges,
    #[serde(rename = "nmea_injector")]
    NmeaInjector,
    #[serde(rename = "available_services")]
    AvailableServices,
    #[serde(rename = "system_information")]
    SystemInformation,
    #[serde(rename = "mavlink_inspector")]
    MavlinkInspector,
    #[serde(rename = "network_test")]
    NetworkTest,
    #[serde(rename = "bag_editor")]
    BagEditor,
    #[serde(rename = "extensions")]
    Extensions,
    #[serde(rename = "extension_manager")]
    ExtensionManager,
    #[serde(rename = "parameter_editor")]
    ParameterEditor,
    #[serde(rename = "zenoh_inspector")]
    ZenohInspector,
    #[serde(rename = "settings")]
    Settings,
}

impl PageId {
    pub const ALL: [PageId; 24] = [
        PageId::Main,
        PageId::Autopilot,
        PageId::VehicleSetup,
        PageId::Pings,
        PageId::LogBrowser,
        PageId::Endpoints,
        PageId::FileBrowser,
        PageId::Disk,
        PageId::Terminal,
        PageId::VersionChooser,
        PageId::VideoManager,
        PageId::Records,
        PageId::Bridges,
        PageId::NmeaInjector,
        PageId::AvailableServices,
        PageId::SystemInformation,
        PageId::MavlinkInspector,
        PageId::NetworkTest,
        PageId::BagEditor,
        PageId::Extensions,
        PageId::ExtensionManager,
        PageId::ParameterEditor,
        PageId::ZenohInspector,
        PageId::Settings,
    ];

    pub const fn as_str(&self) -> &'static str {
        match self {
            PageId::Main => "main",
            PageId::Autopilot => "autopilot",
            PageId::VehicleSetup => "vehicle_setup",
            PageId::Pings => "pings",
            PageId::LogBrowser => "log_browser",
            PageId::Endpoints => "endpoints",
            PageId::FileBrowser => "file_browser",
            PageId::Disk => "disk",
            PageId::Terminal => "terminal",
            PageId::VersionChooser => "version_chooser",
            PageId::VideoManager => "video_manager",
            PageId::Records => "records",
            PageId::Bridges => "bridges",
            PageId::NmeaInjector => "nmea_injector",
            PageId::AvailableServices => "available_services",
            PageId::SystemInformation => "system_information",
            PageId::MavlinkInspector => "mavlink_inspector",
            PageId::NetworkTest => "network_test",
            PageId::BagEditor => "bag_editor",
            PageId::Extensions => "extensions",
            PageId::ExtensionManager => "extension_manager",
            PageId::ParameterEditor => "parameter_editor",
            PageId::ZenohInspector => "zenoh_inspector",
            PageId::Settings => "settings",
        }
    }

    pub fn from_str_id(s: &str) -> Option<PageId> {
        PageId::ALL.into_iter().find(|v| v.as_str() == s)
    }
}

impl fmt::Display for PageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Entity for PageId {
    const ALL: &'static [PageId] = &PageId::ALL;
    fn as_str(&self) -> &'static str {
        PageId::as_str(self)
    }
}
