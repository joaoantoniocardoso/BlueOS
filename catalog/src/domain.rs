use std::fmt;

use schemars::JsonSchema;
use serde::Serialize;

use crate::capability::Aggregate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
pub enum Domain {
    #[serde(rename = "vehicle")]
    Vehicle,
    #[serde(rename = "peripherals")]
    Peripherals,
    #[serde(rename = "onboard_computer")]
    OnboardComputer,
    #[serde(rename = "network")]
    Network,
    #[serde(rename = "blueos_platform")]
    BlueOsPlatform,
    #[serde(rename = "presentation")]
    Presentation,
}

impl Domain {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Domain::Vehicle => "vehicle",
            Domain::Peripherals => "peripherals",
            Domain::OnboardComputer => "onboard_computer",
            Domain::Network => "network",
            Domain::BlueOsPlatform => "blueos_platform",
            Domain::Presentation => "presentation",
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct DomainDef {
    pub id: Domain,
    pub aggregates: &'static [Aggregate],
    pub rationale: &'static str,
}

pub const DOMAINS: &[DomainDef] = &[
    DomainDef {
        id: Domain::Vehicle,
        aggregates: &[
            Aggregate::Autopilot,
            Aggregate::Mavlink,
            Aggregate::Recording,
        ],
        rationale: "Flight controller, MAVLink routing, and vehicle-side recording",
    },
    DomainDef {
        id: Domain::Peripherals,
        aggregates: &[
            Aggregate::Camera,
            Aggregate::SerialBridge,
            Aggregate::Sonar,
            Aggregate::GpsNmea,
        ],
        rationale: "Attached sensors and serial bridges beyond the flight stack",
    },
    DomainDef {
        id: Domain::OnboardComputer,
        aggregates: &[
            Aggregate::HostControl,
            Aggregate::FilesKv,
            Aggregate::Storage,
            Aggregate::ShellAccess,
        ],
        rationale: "Companion-computer host control, files, storage, and shell access",
    },
    DomainDef {
        id: Domain::Network,
        aggregates: &[
            Aggregate::WiredNetwork,
            Aggregate::WirelessNetwork,
            Aggregate::IdentityDiscovery,
            Aggregate::NetDiagnostics,
        ],
        rationale: "Connectivity, LAN identity, and link diagnostics",
    },
    DomainDef {
        id: Domain::BlueOsPlatform,
        aggregates: &[
            Aggregate::Versioning,
            Aggregate::Extensions,
            Aggregate::WebIngress,
            Aggregate::MessageBus,
        ],
        rationale: "OS lifecycle, extensions, HTTP ingress, and inter-service messaging",
    },
    DomainDef {
        id: Domain::Presentation,
        aggregates: &[Aggregate::BrandingUi],
        rationale: "Web UI branding and frontend shell",
    },
];

pub const ALL_AGGREGATES: &[Aggregate] = &[
    Aggregate::Autopilot,
    Aggregate::BrandingUi,
    Aggregate::Camera,
    Aggregate::Extensions,
    Aggregate::FilesKv,
    Aggregate::GpsNmea,
    Aggregate::HostControl,
    Aggregate::IdentityDiscovery,
    Aggregate::Mavlink,
    Aggregate::MessageBus,
    Aggregate::NetDiagnostics,
    Aggregate::Recording,
    Aggregate::SerialBridge,
    Aggregate::ShellAccess,
    Aggregate::Sonar,
    Aggregate::Storage,
    Aggregate::Versioning,
    Aggregate::WebIngress,
    Aggregate::WiredNetwork,
    Aggregate::WirelessNetwork,
];

pub fn domain_of(aggregate: Aggregate) -> Domain {
    aggregate_domain(aggregate).id
}

pub fn aggregate_domain(aggregate: Aggregate) -> &'static DomainDef {
    DOMAINS
        .iter()
        .find(|def| def.aggregates.contains(&aggregate))
        .unwrap_or_else(|| panic!("aggregate {aggregate} has no domain"))
}
