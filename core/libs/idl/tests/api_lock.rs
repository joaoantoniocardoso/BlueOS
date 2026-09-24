use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use blueos_idl_codegen::{
    collect_messages_for_test, field_signature_hash, format_lock_line, is_append_only_evolution,
    parse_lock_line,
};

fn interfaces_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("interfaces")
}

fn lock_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../interfaces/api.lock")
}

fn read_lock() -> BTreeMap<String, (u32, String)> {
    let content = fs::read_to_string(lock_path()).expect("read api.lock");
    content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(|line| {
            let (schema_name, major, hash) = parse_lock_line(line).expect("lock line");
            (schema_name, (major, hash))
        })
        .collect()
}

#[test]
fn api_lock_matches_interfaces() {
    let current = collect_messages_for_test(&interfaces_root());
    let locked = read_lock();
    let updating = std::env::var("BLUEOS_IDL_UPDATE_LOCK").as_deref() == Ok("1");

    if updating {
        let mut lines = Vec::new();
        for record in &current {
            let major = locked
                .get(&record.schema_name)
                .map(|(major, _hash)| *major)
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
        "message count changed; run BLUEOS_IDL_UPDATE_LOCK=1 cargo test -p blueos-idl api_lock_matches_interfaces"
    );
    for record in current {
        let (major, locked_hash) = locked
            .get(&record.schema_name)
            .expect("schema missing from api.lock");
        let current_hash = field_signature_hash(&record.field_signature);
        assert_eq!(
            current_hash, *locked_hash,
            "IDL drift for {} (major {}); run BLUEOS_IDL_UPDATE_LOCK=1 cargo test -p blueos-idl api_lock_matches_interfaces",
            record.schema_name, major
        );
    }
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
