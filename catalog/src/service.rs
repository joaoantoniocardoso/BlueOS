use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::criticality::CriticalityTier;
use crate::edge::Edge;
use crate::id::{CapabilityId, JourneyId, PathRef, ServiceId};
use crate::lifecycle::Lifecycle;
use crate::provenance::{Asserted, AssertedSet};
use crate::resource::Resource;
use crate::state::StateMachine;
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ServiceDefinition {
    pub id: ServiceId,
    pub singleton: Asserted<bool>,
    pub bounded_context: Asserted<String>,
    pub journey_refs: AssertedSet<JourneyId>,
    pub tier: Asserted<CriticalityTier>,
    pub offline_required: Asserted<bool>,
    pub privilege_level: Asserted<PrivilegeLevel>,
    pub dangerous_operations: AssertedSet<DangerousOperation>,
    pub user_confirmation: Asserted<UserConfirmation>,
    pub capabilities: AssertedSet<CapabilityId>,
    pub authorities: AssertedSet<Authority>,
    pub states: AssertedSet<StateMachine>,
    pub edges: AssertedSet<Edge>,
    pub resources: AssertedSet<Resource>,
    pub lifecycle: Lifecycle,
    pub health: Asserted<String>,
    pub is_platform: Asserted<bool>,
    pub api_stable: Asserted<bool>,
    pub permissions_model: Asserted<String>,
    pub failure_modes: AssertedSet<String>,
    pub blast_radius: Asserted<String>,
    pub compatibility_policy: Asserted<String>,
    pub team: Asserted<String>,
    pub adr_refs: AssertedSet<String>,
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
