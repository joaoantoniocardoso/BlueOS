use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct StateMachine {
    pub name: &'static str,
    pub states: &'static [&'static str],
    pub boot_state: &'static str,
    pub degraded_when: &'static [&'static str],
}
