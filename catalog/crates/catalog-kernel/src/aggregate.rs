use std::fmt;

use schemars::JsonSchema;
use serde::Serialize;

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
    #[serde(rename = "message_bus")]
    MessageBus,
    #[serde(rename = "net_diagnostics")]
    NetDiagnostics,
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
    #[serde(rename = "web_ingress")]
    WebIngress,
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
            Aggregate::MessageBus => "message_bus",
            Aggregate::NetDiagnostics => "net_diagnostics",
            Aggregate::Recording => "recording",
            Aggregate::SerialBridge => "serial_bridge",
            Aggregate::ShellAccess => "shell_access",
            Aggregate::Sonar => "sonar",
            Aggregate::Storage => "storage",
            Aggregate::Versioning => "versioning",
            Aggregate::WebIngress => "web_ingress",
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
