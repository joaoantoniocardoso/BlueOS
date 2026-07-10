use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::criticality::CriticalityTier;
use crate::edge::Edge;
use crate::id::{CapabilityId, PathRef, ServiceId};
use crate::lifecycle::Lifecycle;
use crate::provenance::Asserted;
use crate::resource::Resource;
use crate::state::StateMachine;
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ServiceDefinition {
    pub id: ServiceId,
    pub singleton: Asserted<bool>,
    pub bounded_context: Asserted<String>,
    pub user_journeys: Asserted<Vec<String>>,
    pub tier: Asserted<CriticalityTier>,
    pub offline_required: Asserted<bool>,
    pub privilege_level: Asserted<PrivilegeLevel>,
    pub dangerous_operations: Asserted<Vec<DangerousOperation>>,
    pub user_confirmation: Asserted<UserConfirmation>,
    pub capabilities: Asserted<Vec<CapabilityId>>,
    pub authorities: Asserted<Vec<Authority>>,
    pub states: Asserted<Vec<StateMachine>>,
    pub edges: Asserted<Vec<Edge>>,
    pub resources: Asserted<Vec<Resource>>,
    pub lifecycle: Lifecycle,
    pub health: Asserted<String>,
    pub is_platform: Asserted<bool>,
    pub api_stable: Asserted<bool>,
    pub permissions_model: Asserted<String>,
    pub failure_modes: Asserted<Vec<String>>,
    pub blast_radius: Asserted<String>,
    pub compatibility_policy: Asserted<String>,
    pub team: Asserted<String>,
    pub adr_refs: Asserted<Vec<String>>,
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
