//! Shared helpers for `blueos-service` integration tests.

use std::{fs, path::Path};

/// Reads a `u32` from JSON at `path` under the `field` key path (`domain.value` style).
pub(crate) fn durable_u32_from_json(path: &Path, field: &str) -> Option<u32> {
    let raw = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let number = value.pointer(field)?.as_u64()?;
    u32::try_from(number).ok()
}
