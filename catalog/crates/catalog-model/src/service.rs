use schemars::JsonSchema;
use serde::Serialize;

use catalog_kernel::{
    criticality::CriticalityTier,
    id::{capability::CapabilityId, journey::JourneyId, refs::PathRef, service::ServiceId},
    provenance::{Asserted, AssertedSet},
};

use crate::edge::Connection;
use crate::lifecycle::Lifecycle;
use crate::observed::ObservedFacts;
use crate::resource::Resource;
use crate::runtime::RuntimeFacts;
use crate::state::StateMachine;
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Service {
    pub id: ServiceId,
    pub observed: ObservedFacts,
    pub definition: ServiceJudgment,
    pub runtime: RuntimeFacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ServiceJudgment {
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
    pub edges: AssertedSet<Connection>,
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
