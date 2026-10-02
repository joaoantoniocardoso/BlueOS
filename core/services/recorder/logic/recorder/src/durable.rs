//! Persisted Recorder state (D-28).

/// Persisted Recorder state; repair progress lives in Jobs, not here.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecorderDurableState;
