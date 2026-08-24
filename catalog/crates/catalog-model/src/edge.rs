use schemars::JsonSchema;
use serde::Serialize;

use catalog_kernel::id::service::ServiceId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Connection {
    pub from: ServiceId,
    pub to: ServiceId,
    pub via: Bus,
    pub sync: SyncMode,
    pub endpoint: &'static str,
    pub purpose: &'static str,
    pub required_at_boot: bool,
    pub failure_impact: FailureImpact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Bus {
    Rest,
    Zenoh,
    Mavlink,
    Websocket,
    HttpStream,
    Subprocess,
    File,
    Settings,
    Hardware,
    Docker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    Sync,
    Async,
    FireAndForget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureImpact {
    None,
    Degraded,
    ServiceUnavailable,
    VehicleUnsafe,
}
