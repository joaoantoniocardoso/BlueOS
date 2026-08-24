use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Evidence {
    pub file: &'static str,
    pub line: u32,
    pub anchor: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Observed<T> {
    Known { value: T, evidence: Evidence },
    Unknown { reason: &'static str },
}

impl<T> Observed<T> {
    pub const fn known(value: T, evidence: Evidence) -> Self {
        Self::Known { value, evidence }
    }

    pub const fn unknown(reason: &'static str) -> Self {
        Self::Unknown { reason }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Asserted<T> {
    Established { value: T, rationale: &'static str },
    Unknown { reason: &'static str },
}

impl<T> Asserted<T> {
    pub const fn established(value: T, rationale: &'static str) -> Self {
        Self::Established { value, rationale }
    }

    pub const fn unknown(reason: &'static str) -> Self {
        Self::Unknown { reason }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }
}

// Each collection item carries its own file:line evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Evidenced<T> {
    pub value: T,
    pub evidence: Evidence,
}

impl<T> Evidenced<T> {
    pub const fn new(value: T, evidence: Evidence) -> Self {
        Self { value, evidence }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ObservedSet<T: 'static> {
    Known { items: &'static [Evidenced<T>] },
    Unknown { reason: &'static str },
}

impl<T: 'static> ObservedSet<T> {
    pub const fn known(items: &'static [Evidenced<T>]) -> Self {
        Self::Known { items }
    }

    pub const fn unknown(reason: &'static str) -> Self {
        Self::Unknown { reason }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }
}

// Each collection item carries its own judgment rationale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Rationaled<T> {
    pub value: T,
    pub rationale: &'static str,
}

impl<T> Rationaled<T> {
    pub const fn new(value: T, rationale: &'static str) -> Self {
        Self { value, rationale }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AssertedSet<T: 'static> {
    Established { items: &'static [Rationaled<T>] },
    Unknown { reason: &'static str },
}

impl<T: 'static> AssertedSet<T> {
    pub const fn established(items: &'static [Rationaled<T>]) -> Self {
        Self::Established { items }
    }

    pub const fn unknown(reason: &'static str) -> Self {
        Self::Unknown { reason }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Source(Evidence),
    Doc {
        file: &'static str,
        line: u32,
        anchor: &'static str,
    },
    Runtime {
        capture: &'static str,
        environment: &'static str,
    },
    Asserted {
        rationale: &'static str,
    },
}

impl Provenance {
    pub const fn source(file: &'static str, line: u32, anchor: &'static str) -> Self {
        Self::Source(Evidence { file, line, anchor })
    }

    pub const fn doc(file: &'static str, line: u32, anchor: &'static str) -> Self {
        Self::Doc { file, line, anchor }
    }

    pub const fn runtime(capture: &'static str, environment: &'static str) -> Self {
        Self::Runtime {
            capture,
            environment,
        }
    }

    pub const fn asserted(rationale: &'static str) -> Self {
        Self::Asserted { rationale }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Grounded<T> {
    Known { value: T, provenance: Provenance },
    Unknown { reason: &'static str },
}

impl<T> Grounded<T> {
    pub const fn known(value: T, provenance: Provenance) -> Self {
        Self::Known { value, provenance }
    }

    pub const fn unknown(reason: &'static str) -> Self {
        Self::Unknown { reason }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct GroundedItem<T> {
    pub value: T,
    pub provenance: Provenance,
}

impl<T> GroundedItem<T> {
    pub const fn new(value: T, provenance: Provenance) -> Self {
        Self { value, provenance }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum GroundedSet<T: 'static> {
    Known { items: &'static [GroundedItem<T>] },
    Unknown { reason: &'static str },
}

impl<T: 'static> GroundedSet<T> {
    pub const fn known(items: &'static [GroundedItem<T>]) -> Self {
        Self::Known { items }
    }

    pub const fn unknown(reason: &'static str) -> Self {
        Self::Unknown { reason }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }
}
