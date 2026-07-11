use schemars::JsonSchema;
use serde::Serialize;

use crate::id::PathRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Resource {
    pub path: PathRef,
    pub ownership: ResourceOwnership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResourceOwnership {
    Exclusive,
    SharedRead,
    SharedWrite,
}
