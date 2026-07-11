use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::{CapabilityId, JourneyId, PathRef, ServiceId};
use crate::provenance::{Grounded, GroundedSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct UserJourney {
    pub id: JourneyId,
    pub summary: Grounded<String>,
    pub visibility: Grounded<Visibility>,
    pub services: GroundedSet<ServiceId>,
    pub capability_refs: GroundedSet<CapabilityId>,
    pub preconditions: GroundedSet<Precondition>,
    pub steps: GroundedSet<JourneyStep>,
    pub chains_from: Option<JourneyId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct JourneyStep {
    pub actor: Actor,
    pub description: String,
    pub route: Option<Grounded<RouteRef>>,
    pub outcome: Option<Grounded<StepOutcome>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RouteRef {
    pub service: ServiceId,
    pub method: HttpMethod,
    pub path: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StepOutcome {
    pub expected_status: Option<u16>,
    pub body_predicate: Option<String>,
    pub transition: Option<StateTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    Operator,
    Service(ServiceId),
    Subprocess(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Default,
    Advanced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Precondition {
    ServiceState { service: ServiceId, state: String },
    Network(NetworkState),
    HardwarePresent(String),
    ConfigClean(PathRef),
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NetworkState {
    Online,
    Offline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StateTransition {
    pub machine: String,
    pub from: String,
    pub to: String,
}
