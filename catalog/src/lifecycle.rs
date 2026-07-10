use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::ServiceId;
use crate::provenance::Asserted;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Lifecycle {
    pub triggers: Asserted<Vec<String>>,
    pub ordered_after: Asserted<Vec<ServiceId>>,
    pub ordered_before: Asserted<Vec<ServiceId>>,
    pub shutdown: Asserted<String>,
    pub upgrade_behavior: Asserted<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedLifecycle {
    pub triggers: Vec<String>,
    pub ordered_after: Vec<ServiceId>,
    pub ordered_before: Vec<ServiceId>,
}
