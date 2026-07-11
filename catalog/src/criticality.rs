use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CriticalityTier {
    VehicleCritical,
    Important,
    Auxiliary,
    Optional,
}
