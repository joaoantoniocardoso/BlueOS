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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Asserted<T> {
    Established { value: T, rationale: String },
    Unknown { reason: String },
}

impl<T> Asserted<T> {
    pub fn established(value: T, rationale: impl Into<String>) -> Self {
        Self::Established {
            value,
            rationale: rationale.into(),
        }
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

// Each collection item carries its own file:line evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Evidenced<T> {
    pub value: T,
    pub evidence: Evidence,
}

impl<T> Evidenced<T> {
    pub fn new(value: T, evidence: Evidence) -> Self {
        Self { value, evidence }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ObservedSet<T> {
    Known { items: Vec<Evidenced<T>> },
    Unknown { reason: String },
}

impl<T> ObservedSet<T> {
    pub fn known(items: Vec<Evidenced<T>>) -> Self {
        Self::Known { items }
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

// Each collection item carries its own judgment rationale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Rationaled<T> {
    pub value: T,
    pub rationale: String,
}

impl<T> Rationaled<T> {
    pub fn new(value: T, rationale: impl Into<String>) -> Self {
        Self {
            value,
            rationale: rationale.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AssertedSet<T> {
    Established { items: Vec<Rationaled<T>> },
    Unknown { reason: String },
}

impl<T> AssertedSet<T> {
    pub fn established(items: Vec<Rationaled<T>>) -> Self {
        Self::Established { items }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Source(Evidence),
    Doc {
        file: String,
        line: u32,
    },
    Runtime {
        capture: String,
        environment: String,
    },
    Asserted {
        rationale: String,
    },
}

impl Provenance {
    pub fn source(file: impl Into<String>, line: u32) -> Self {
        Self::Source(Evidence {
            file: file.into(),
            line,
        })
    }

    pub fn doc(file: impl Into<String>, line: u32) -> Self {
        Self::Doc {
            file: file.into(),
            line,
        }
    }

    pub fn runtime(capture: impl Into<String>, environment: impl Into<String>) -> Self {
        Self::Runtime {
            capture: capture.into(),
            environment: environment.into(),
        }
    }

    pub fn asserted(rationale: impl Into<String>) -> Self {
        Self::Asserted {
            rationale: rationale.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Grounded<T> {
    Known { value: T, provenance: Provenance },
    Unknown { reason: String },
}

impl<T> Grounded<T> {
    pub fn known(value: T, provenance: Provenance) -> Self {
        Self::Known { value, provenance }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GroundedItem<T> {
    pub value: T,
    pub provenance: Provenance,
}

impl<T> GroundedItem<T> {
    pub fn new(value: T, provenance: Provenance) -> Self {
        Self { value, provenance }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum GroundedSet<T> {
    Known { items: Vec<GroundedItem<T>> },
    Unknown { reason: String },
}

impl<T> GroundedSet<T> {
    pub fn known(items: Vec<GroundedItem<T>>) -> Self {
        Self::Known { items }
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
