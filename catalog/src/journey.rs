use schemars::JsonSchema;
use serde::Serialize;

use crate::id::{CapabilityId, JourneyId, PathRef, ServiceId};
use crate::provenance::{Grounded, GroundedSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct UserJourney {
    pub id: JourneyId,
    pub summary: Grounded<&'static str>,
    pub visibility: Grounded<Visibility>,
    pub services: GroundedSet<ServiceId>,
    pub capability_refs: GroundedSet<CapabilityId>,
    pub preconditions: GroundedSet<Precondition>,
    pub steps: GroundedSet<JourneyStep>,
    pub chains_from: Option<JourneyId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct JourneyStep {
    pub actor: Actor,
    pub description: &'static str,
    pub route: Option<Grounded<RouteRef>>,
    pub outcome: Option<Grounded<StepOutcome>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct RouteRef {
    pub service: ServiceId,
    pub method: HttpMethod,
    pub path: &'static str,
    pub version: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct StepOutcome {
    pub expected_status: Option<u16>,
    pub body_predicate: Option<&'static str>,
    pub transition: Option<StateTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    Operator,
    Service(ServiceId),
    Subprocess(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Default,
    Advanced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Precondition {
    ServiceState {
        service: ServiceId,
        state: &'static str,
    },
    Network(NetworkState),
    HardwarePresent(&'static str),
    ConfigClean(PathRef),
    Other(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NetworkState {
    Online,
    Offline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct StateTransition {
    pub machine: &'static str,
    pub from: &'static str,
    pub to: &'static str,
}
