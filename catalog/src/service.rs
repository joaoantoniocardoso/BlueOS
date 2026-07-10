use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::criticality::CriticalityTier;
use crate::edge::Edge;
use crate::id::{CapabilityId, PathRef, ServiceId};
use crate::lifecycle::Lifecycle;
use crate::provenance::Observed;
use crate::resource::Resource;
use crate::state::StateMachine;
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ServiceDefinition {
    pub id: ServiceId,
    pub singleton: Observed<bool>,
    pub bounded_context: Observed<String>,
    pub user_journeys: Observed<Vec<String>>,
    pub tier: Observed<CriticalityTier>,
    pub offline_required: Observed<bool>,
    pub privilege_level: Observed<PrivilegeLevel>,
    pub dangerous_operations: Observed<Vec<DangerousOperation>>,
    pub user_confirmation: Observed<UserConfirmation>,
    pub capabilities: Observed<Vec<CapabilityId>>,
    pub authorities: Observed<Vec<Authority>>,
    pub states: Observed<Vec<StateMachine>>,
    pub edges: Observed<Vec<Edge>>,
    pub resources: Observed<Vec<Resource>>,
    pub lifecycle: Lifecycle,
    pub health: Observed<String>,
    pub is_platform: Observed<bool>,
    pub api_stable: Observed<bool>,
    pub permissions_model: Observed<String>,
    pub failure_modes: Observed<Vec<String>>,
    pub blast_radius: Observed<String>,
    pub compatibility_policy: Observed<String>,
    pub team: Observed<String>,
    pub adr_refs: Observed<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Authority {
    MavlinkRouterOwner,
    NginxProxy,
    ZenohBroker,
    UserdataWriter(PathRef),
    HardwareExclusive(PathRef),
    Other(String),
}
