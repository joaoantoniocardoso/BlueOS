use schemars::JsonSchema;
use serde::Serialize;

use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::GroundedSet;

use crate::journey::RouteRef;

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct RuntimeFacts {
    pub service: ServiceId,
    pub state_contracts: GroundedSet<StateContract>,
    pub slo_baselines: GroundedSet<SloBaseline>,
    pub resource_usage: GroundedSet<ResourceUsage>,
    pub platform_matrix: GroundedSet<PlatformBehavior>,
    pub settings_mutations: GroundedSet<SettingsMutation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct StateContract {
    pub machine: &'static str,
    pub state: &'static str,
    pub route: RouteRef,
    pub status: u16,
    pub body_predicate: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct SloBaseline {
    pub route: RouteRef,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub sample_size: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct ResourceUsage {
    pub condition: &'static str,
    pub cpu_pct: Distribution,
    pub rss_mb: Distribution,
    pub samples: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Distribution {
    pub mean: f64,
    pub median: f64,
    pub p95: f64,
    pub min: f64,
    pub max: f64,
    pub sd: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct PlatformBehavior {
    pub platform: &'static str,
    pub firmware: Option<&'static str>,
    pub notes: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct SettingsMutation {
    pub trigger: &'static str,
    pub keys_changed: &'static [&'static str],
}
