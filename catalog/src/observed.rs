use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::{PathRef, PortRef, ServiceId};
use crate::interface::Interface;
use crate::lifecycle::ObservedLifecycle;
use crate::provenance::Observed;
use crate::resource::Resource;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedFacts {
    pub id: ServiceId,
    pub aliases: Observed<Vec<String>>,
    pub kind: Observed<ServiceKind>,
    pub entrypoint: Observed<String>,
    pub tmux_name: Observed<String>,
    pub startup_tier: Observed<StartupTier>,
    pub resource_limits: Observed<ResourceLimits>,
    pub nice: Observed<i32>,
    pub run_as: Observed<String>,
    pub nginx_prefixes: Observed<Vec<PathRef>>,
    pub listen: Observed<Vec<PortRef>>,
    pub git_path: Observed<PathRef>,
    pub interfaces: Observed<Vec<Interface>>,
    pub resources: Observed<Vec<Resource>>,
    pub lifecycle: Observed<ObservedLifecycle>,
    pub logs_path: Observed<PathRef>,
    pub zenoh_log_topic: Observed<String>,
    pub sentry: Observed<bool>,
    pub openapi_refs: Observed<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ServiceKind {
    PythonService,
    Binary,
    Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StartupTier {
    Priority,
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResourceLimits {
    pub memory_mb: Option<u32>,
    pub cpu_percent: Option<u32>,
    pub io_weight: Option<u32>,
}
