use schemars::JsonSchema;
use serde::Serialize;

use crate::id::ServiceId;
use crate::provenance::Asserted;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Lifecycle {
    pub triggers: Asserted<&'static [&'static str]>,
    pub ordered_after: Asserted<&'static [ServiceId]>,
    pub ordered_before: Asserted<&'static [ServiceId]>,
    pub shutdown: Asserted<&'static str>,
    pub upgrade_behavior: Asserted<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ObservedLifecycle {
    pub triggers: &'static [&'static str],
    pub ordered_after: &'static [ServiceId],
    pub ordered_before: &'static [ServiceId],
}
