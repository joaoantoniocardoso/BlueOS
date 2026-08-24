use schemars::JsonSchema;
use serde::Serialize;

use crate::id::{PathRef, PortRef, ServiceId};
use crate::interface::PortKind;
use crate::lifecycle::ObservedLifecycle;
use crate::provenance::{Observed, ObservedSet};
use crate::resource::Resource;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ObservedFacts {
    pub id: ServiceId,
    pub aliases: ObservedSet<&'static str>,
    pub kind: Observed<ServiceKind>,
    pub entrypoint: Observed<&'static str>,
    pub tmux_name: Observed<&'static str>,
    pub startup_tier: Observed<StartupTier>,
    pub resource_limits: Observed<ResourceLimits>,
    pub nice: Observed<i32>,
    pub run_as: Observed<&'static str>,
    pub nginx_prefixes: ObservedSet<PathRef>,
    pub listen: ObservedSet<PortRef>,
    pub git_path: Observed<PathRef>,
    pub interfaces: ObservedSet<PortKind>,
    pub resources: ObservedSet<Resource>,
    pub lifecycle: Observed<ObservedLifecycle>,
    pub logs_path: Observed<PathRef>,
    pub zenoh_log_topic: Observed<&'static str>,
    pub sentry: Observed<bool>,
    pub openapi_refs: ObservedSet<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ServiceKind {
    PythonService,
    Binary,
    Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StartupTier {
    Priority,
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ResourceLimits {
    pub memory_mb: Option<u32>,
    pub cpu_percent: Option<u32>,
    pub io_weight: Option<u32>,
}
