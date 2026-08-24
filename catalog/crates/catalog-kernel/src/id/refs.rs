use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct TcpPort(pub u16);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct PathRef(pub &'static str);

// Externally tagged: internal tagging (`tag = "kind"`) cannot serialize a newtype
// variant wrapping a primitive (`Literal(u16)`). Serializes as {"literal": 8000} / {"env": "VAR"}.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PortRef {
    Literal(u16),
    Env(&'static str),
}
