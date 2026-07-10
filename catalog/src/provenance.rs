use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Evidence {
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Observed<T> {
    Known { value: T, evidence: Evidence },
    Unknown { reason: String },
}

impl<T> Observed<T> {
    pub fn known(value: T, evidence: Evidence) -> Self {
        Self::Known { value, evidence }
    }

    pub fn unknown(reason: impl Into<String>) -> Self {
        Self::Unknown {
            reason: reason.into(),
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }
}
