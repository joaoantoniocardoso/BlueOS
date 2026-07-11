use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Identity of a cataloged BlueOS service. Closed set of the 26 processes launched by
/// `core/start-blueos-core`. Each variant serializes to its canonical id string (explicit
/// `rename` on every variant so the JSON is exact and independent of `rename_all` heuristics).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct CapabilityId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct JourneyId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Port(pub u16);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct PathRef(pub String);

// Externally tagged: internal tagging (`tag = "kind"`) cannot serialize a newtype
// variant wrapping a primitive (`Literal(u16)`). Serializes as {"literal": 8000} / {"env": "VAR"}.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PortRef {
    Literal(u16),
    Env(String),
}
