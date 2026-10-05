//! `api.lock` drift and schema-evolution gate tests (D-06).

use std::{collections::BTreeMap, fs, path::PathBuf};

use blueos_idl_codegen::{
    check_message_lock, collect_messages_for_test, endpoints::collect_endpoint_lock_lines,
    explain_endpoint_lock_mismatch, explain_lock_mismatch, field_signature_hash, format_lock_line,
    is_append_only_evolution, parse_lock_line,
};

fn interfaces_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("interfaces")
}

fn core_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("libs")
        .parent()
        .expect("core root")
        .to_path_buf()
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

fn message_lock(lock: &BTreeMap<String, (u32, String)>) -> BTreeMap<String, (u32, String)> {
    lock.iter()
        .filter(|(name, _)| !name.starts_with("blueos/v1/"))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

fn endpoint_lock(lock: &BTreeMap<String, (u32, String)>) -> BTreeMap<String, (u32, String)> {
    lock.iter()
        .filter(|(name, _)| name.starts_with("blueos/v1/"))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

#[test]
fn api_lock_matches_interfaces() {
    let current = collect_messages_for_test(&interfaces_root()).expect("parse interfaces");
    let locked = message_lock(&read_lock());
    let updating = std::env::var("BLUEOS_IDL_UPDATE_LOCK").as_deref() == Ok("1");

    if updating {
        write_lock_file();
        return;
    }

    if let Err(reason) = check_message_lock(&locked, &current) {
        panic!("{reason}");
    }
}

#[test]
fn api_lock_matches_endpoint_manifests() {
    let messages = collect_messages_for_test(&interfaces_root())
        .expect("parse interfaces")
        .into_iter()
        .map(|record| record.schema_name)
        .collect();
    let current =
        collect_endpoint_lock_lines(&core_root(), &messages).expect("collect endpoint keys");
    let current: BTreeMap<String, (u32, String)> = current
        .into_iter()
        .map(|line| {
            let (key, major, signature) = parse_lock_line(&line).expect("lock line");
            (key, (major, signature))
        })
        .collect();
    let locked = endpoint_lock(&read_lock());
    let updating = std::env::var("BLUEOS_IDL_UPDATE_LOCK").as_deref() == Ok("1");

    if updating {
        write_lock_file();
        return;
    }

    assert_eq!(
        locked.len(),
        current.len(),
        "endpoint key count changed; run: cargo run -p blueos-idl-codegen --bin blueos-idl-print-lock > core/libs/idl/api.lock"
    );
    for (key, (major, current_signature)) in &current {
        let (locked_major, locked_signature) =
            locked.get(key).expect("endpoint key missing from api.lock");
        assert_eq!(*locked_major, *major);
        if locked_signature == current_signature {
            continue;
        }
        panic!(
            "{}",
            explain_endpoint_lock_mismatch(key, *major, locked_signature, current_signature)
        );
    }
}

fn write_lock_file() {
    let current = collect_messages_for_test(&interfaces_root()).expect("parse interfaces");
    let messages = current
        .iter()
        .map(|record| record.schema_name.clone())
        .collect();
    let previous = read_lock();
    let mut lines = Vec::new();
    for record in &current {
        let major = previous
            .get(&record.schema_name)
            .map(|(major, _field_signature)| *major)
            .unwrap_or(1);
        lines.push(format_lock_line(
            &record.schema_name,
            major,
            &record.field_signature,
        ));
    }
    for line in collect_endpoint_lock_lines(&core_root(), &messages).expect("collect endpoint keys")
    {
        let (key, _major, _signature) = parse_lock_line(&line).expect("lock line");
        let major = previous.get(&key).map(|(major, _)| *major).unwrap_or(1);
        let signature = parse_lock_line(&line).expect("lock line").2;
        lines.push(format_lock_line(&key, major, &signature));
    }
    lines.sort();
    fs::write(lock_path(), format!("{}\n", lines.join("\n"))).expect("write api.lock");
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

#[test]
fn endpoint_lock_requires_major_bump_for_signature_change() {
    let message = explain_endpoint_lock_mismatch(
        "blueos/v1/tank/command/Drain",
        1,
        "type=blueos_example_msgs/srv/Level",
        "type=blueos_example_msgs/action/SetLevel",
    );
    assert!(message.contains("endpoint API change"));
}
