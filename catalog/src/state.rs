use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachine {
    pub name: String,
    pub states: Vec<String>,
    pub boot_state: String,
    pub degraded_when: Vec<String>,
}
