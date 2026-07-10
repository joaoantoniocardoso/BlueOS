use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::ServiceId;
use crate::provenance::Observed;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Lifecycle {
    pub triggers: Observed<Vec<String>>,
    pub ordered_after: Observed<Vec<ServiceId>>,
    pub ordered_before: Observed<Vec<ServiceId>>,
    pub shutdown: Observed<String>,
    pub upgrade_behavior: Observed<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedLifecycle {
    pub triggers: Vec<String>,
    pub ordered_after: Vec<ServiceId>,
    pub ordered_before: Vec<ServiceId>,
}
