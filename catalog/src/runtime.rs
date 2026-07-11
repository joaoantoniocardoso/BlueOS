use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::ServiceId;
use crate::journey::RouteRef;
use crate::provenance::GroundedSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeFacts {
    pub service: ServiceId,
    pub state_contracts: GroundedSet<StateContract>,
    pub slo_baselines: GroundedSet<SloBaseline>,
    pub resource_usage: GroundedSet<ResourceUsage>,
    pub platform_matrix: GroundedSet<PlatformBehavior>,
    pub settings_mutations: GroundedSet<SettingsMutation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateContract {
    pub machine: String,
    pub state: String,
    pub route: RouteRef,
    pub status: u16,
    pub body_predicate: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SloBaseline {
    pub route: RouteRef,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub sample_size: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ResourceUsage {
    pub condition: String,
    pub cpu_pct: Distribution,
    pub rss_mb: Distribution,
    pub samples: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Distribution {
    pub mean: f64,
    pub median: f64,
    pub p95: f64,
    pub min: f64,
    pub max: f64,
    pub sd: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PlatformBehavior {
    pub platform: String,
    pub firmware: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SettingsMutation {
    pub trigger: String,
    pub keys_changed: Vec<String>,
}
