use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::ServiceId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Edge {
    pub from: ServiceId,
    pub to: ServiceId,
    pub via: Bus,
    pub sync: SyncMode,
    pub endpoint: String,
    pub purpose: String,
    pub required_at_boot: bool,
    pub failure_impact: FailureImpact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    Sync,
    Async,
    FireAndForget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureImpact {
    None,
    Degraded,
    ServiceUnavailable,
    VehicleUnsafe,
}
