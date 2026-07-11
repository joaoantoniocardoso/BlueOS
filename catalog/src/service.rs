use schemars::JsonSchema;
use serde::Serialize;

use crate::criticality::CriticalityTier;
use crate::edge::Edge;
use crate::id::{CapabilityId, JourneyId, PathRef, ServiceId};
use crate::lifecycle::Lifecycle;
use crate::provenance::{Asserted, AssertedSet};
use crate::resource::Resource;
use crate::state::StateMachine;
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ServiceDefinition {
    pub id: ServiceId,
    pub singleton: Asserted<bool>,
    pub bounded_context: Asserted<&'static str>,
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
    pub health: Asserted<&'static str>,
    pub is_platform: Asserted<bool>,
    pub api_stable: Asserted<bool>,
    pub permissions_model: Asserted<&'static str>,
    pub failure_modes: AssertedSet<&'static str>,
    pub blast_radius: Asserted<&'static str>,
    pub compatibility_policy: Asserted<&'static str>,
    pub team: Asserted<&'static str>,
    pub adr_refs: AssertedSet<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Authority {
    MavlinkRouterOwner,
    NginxProxy,
    ZenohBroker,
    UserdataWriter(PathRef),
    HardwareExclusive(PathRef),
    Other(&'static str),
}
