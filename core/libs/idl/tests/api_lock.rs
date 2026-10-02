//! `api.lock` drift and schema-evolution gate tests (D-06).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use blueos_idl_codegen::{
    collect_messages_for_test, explain_lock_mismatch, field_signature_hash, format_lock_line,
    frozen_message_schemas, is_append_only_evolution, parse_lock_line,
};

fn interfaces_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("interfaces")
}

fn lock_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("api.lock")
}

fn read_lock() -> BTreeMap<String, (u32, String)> {
    let content = fs::read_to_string(lock_path()).expect("read api.lock");
    content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(|line| {
            let (schema_name, major, field_signature) = parse_lock_line(line).expect("lock line");
            (schema_name, (major, field_signature))
        })
        .collect()
}

#[test]
fn api_lock_matches_interfaces() {
    let current = collect_messages_for_test(&interfaces_root());
    let frozen = frozen_message_schemas(&current);
    let locked = read_lock();
    let updating = std::env::var("BLUEOS_IDL_UPDATE_LOCK").as_deref() == Ok("1");

    if updating {
        let mut lines = Vec::new();
        for record in &current {
            let major = locked
                .get(&record.schema_name)
                .map(|(major, _field_signature)| *major)
                .unwrap_or(1);
            lines.push(format_lock_line(
                &record.schema_name,
                major,
                &record.field_signature,
            ));
        }
        lines.sort();
        fs::write(lock_path(), format!("{}\n", lines.join("\n"))).expect("write api.lock");
        return;
    }

    assert_eq!(
        locked.len(),
        current.len(),
        "message count changed; run: cargo run -p blueos-idl-codegen --bin blueos-idl-print-lock > core/libs/idl/api.lock"
    );
    for record in current {
        let (major, locked_signature) = locked
            .get(&record.schema_name)
            .expect("schema missing from api.lock");
        let current_signature = &record.field_signature;
        if locked_signature == current_signature {
            continue;
        }
        let frozen = frozen.contains(&record.schema_name);
        panic!(
            "{}",
            explain_lock_mismatch(
                &record.schema_name,
                *major,
                locked_signature,
                current_signature,
                frozen,
            )
        );
    }
}

#[test]
fn draft1_command_ack_field_signature_hash() {
    let signature = "accepted:bool;job_id:uint64;reason:string";
    assert_eq!(
        field_signature_hash(signature),
        "2d420dd01bb5d513cbcdd12bc4621ce4db2d06e949e43f2a0edc2e2e4ec28e08"
    );
}

#[test]
fn evolution_gate_rejects_removed_field() {
    assert!(!is_append_only_evolution(
        "accepted:bool;job_id:uint64;reason:string",
        "accepted:bool;job_id:uint64"
    ));
}

#[test]
fn evolution_gate_rejects_reordered_field() {
    assert!(!is_append_only_evolution(
        "accepted:bool;job_id:uint64;reason:string",
        "job_id:uint64;accepted:bool;reason:string"
    ));
}

#[test]
fn evolution_gate_rejects_retyped_field() {
    assert!(!is_append_only_evolution(
        "accepted:bool;job_id:uint64",
        "accepted:bool;job_id:uint32"
    ));
}

#[test]
fn evolution_gate_accepts_appended_field() {
    assert!(is_append_only_evolution(
        "accepted:bool;job_id:uint64",
        "accepted:bool;job_id:uint64;reason:string"
    ));
}

#[test]
fn frozen_message_rejects_append_only_signature_change() {
    let message = explain_lock_mismatch(
        "blueos_msgs/msg/JobStatus",
        1,
        "job_id:uint64",
        "job_id:uint64;detail:string",
        true,
    );
    assert!(message.contains("frozen"));
}

#[test]
fn top_level_append_only_suggests_lock_refresh() {
    let message = explain_lock_mismatch(
        "blueos_msgs/msg/CommandAck",
        1,
        "accepted:bool;job_id:uint64",
        "accepted:bool;job_id:uint64;reason:string",
        false,
    );
    assert!(message.contains("append-only"));
}

#[test]
fn breaking_change_requires_major_bump_message() {
    let message = explain_lock_mismatch(
        "blueos_msgs/msg/CommandAck",
        1,
        "accepted:bool;job_id:uint64;reason:string",
        "accepted:bool;job_id:uint32;reason:string",
        false,
    );
    assert!(message.contains("breaking"));
}
