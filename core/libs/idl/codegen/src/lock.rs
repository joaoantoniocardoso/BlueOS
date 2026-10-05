//! `api.lock` helpers for message and endpoint evolution (D-06).

use alloc::collections::{BTreeMap, BTreeSet};

use crate::{collect::MessageRecord, schema::hash_hex};

/// The SHA-256 of `field_signature`, in hex.
// qual:test_helper used only by blueos-idl api.lock tests
pub fn field_signature_hash(field_signature: &str) -> String {
    hash_hex(field_signature)
}

/// Compares the message lines of `api.lock` (schema name to major version and field signature) with the messages
/// `current` has, every `.srv` and `.action` part included (D-06). The error says why the lock cannot stay as-is.
// qual:test_helper used only by blueos-idl and codegen integration tests
pub fn check_message_lock(
    locked: &BTreeMap<String, (u32, String)>,
    current: &[MessageRecord],
) -> Result<(), String> {
    if locked.len() != current.len() {
        return Err(
            "message count changed; run: cargo run -p blueos-idl-codegen --bin blueos-idl-print-lock > core/libs/idl/api.lock"
                .to_owned(),
        );
    }
    let frozen = frozen_message_schemas(current);
    for record in current {
        let Some((major, locked_signature)) = locked.get(&record.schema_name) else {
            return Err(format!("{} is missing from api.lock", record.schema_name));
        };
        if *locked_signature != record.field_signature {
            return Err(explain_lock_mismatch(
                &record.schema_name,
                *major,
                locked_signature,
                &record.field_signature,
                frozen.contains(&record.schema_name),
            ));
        }
    }
    Ok(())
}

/// Message types referenced as a field or sequence element of another message (D-06).
pub fn frozen_message_schemas(records: &[MessageRecord]) -> BTreeSet<String> {
    records
        .iter()
        .flat_map(|record| record.dependencies.iter().cloned())
        .collect()
}

/// When `locked_signature` and `current_signature` differ, returns why the lock cannot stay as-is.
pub fn explain_lock_mismatch(
    schema_name: &str,
    major: u32,
    locked_signature: &str,
    current_signature: &str,
    frozen: bool,
) -> String {
    if frozen {
        return format!(
            "{schema_name} is frozen (nested in another message); field signature cannot change (major {major})"
        );
    }
    if is_append_only_evolution(locked_signature, current_signature) {
        return format!(
            "{schema_name} (major {major}) is append-only; refresh api.lock (cargo run -p blueos-idl-codegen --bin blueos-idl-print-lock)"
        );
    }
    format!(
        "{schema_name} (major {major}) breaking IDL change; bump major in api.lock and refresh the lock"
    )
}

/// Whether the field signature `current` keeps every field of `previous`, in order, and only adds fields at the end
/// (D-06).
pub fn is_append_only_evolution(previous: &str, current: &str) -> bool {
    if previous == current {
        return true;
    }
    if current.len() <= previous.len() {
        return false;
    }
    if !current.starts_with(previous) {
        return false;
    }
    let suffix = current.strip_prefix(previous).unwrap_or("");
    suffix.is_empty() || suffix.starts_with(';')
}

/// Splits an `api.lock` line into its schema name or endpoint key, its major version and its signature, which is
/// empty for a message without fields. `None` when the line has no major version.
// qual:test_helper used only by blueos-idl api.lock tests
pub fn parse_lock_line(line: &str) -> Option<(String, u32, String)> {
    let mut parts = line.split_whitespace();
    let schema_name = parts.next()?.to_string();
    let major = parts.next()?.parse().ok()?;
    let field_signature = parts.next().unwrap_or("").to_string();
    Some((schema_name, major, field_signature))
}

/// The `api.lock` line of a schema name or endpoint key, the inverse of [`parse_lock_line`].
pub fn format_lock_line(schema_name: &str, major: u32, field_signature: &str) -> String {
    if field_signature.is_empty() {
        format!("{schema_name} {major}")
    } else {
        format!("{schema_name} {major} {field_signature}")
    }
}

/// When an endpoint key's locked and current Message signatures differ, returns why the lock cannot stay as-is.
// qual:test_helper used only by blueos-idl api.lock tests
pub fn explain_endpoint_lock_mismatch(
    key: &str,
    major: u32,
    locked_signature: &str,
    current_signature: &str,
) -> String {
    if locked_signature == current_signature {
        return format!("{key} (major {major}) signatures match");
    }
    format!(
        "{key} (major {major}) endpoint API change; bump major in api.lock and refresh the lock"
    )
}
