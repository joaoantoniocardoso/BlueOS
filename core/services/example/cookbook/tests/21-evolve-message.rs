//! Evolve a published message under the append-only rule (D-06).
//!
//! Workflow:
//! - Append the new field at the end of the `.msg` file (never reorder or remove fields).
//! - Regenerate IDL and clients from `core/`: `cargo run -p blueos-idl-codegen -- --write`
//!   (this refreshes `core/libs/idl/api.lock` with the rest of the generated output).
//! - Pre-push runs `cargo test -p blueos-idl --test api_lock` ("Checking api.lock.."); a reorder or
//!   removal fails the `evolution_gate_*` cases there even when the field signatures look similar.
//!
//! The tests below exercise `is_append_only_evolution`, the same gate the lock uses for signatures.

use blueos_idl_codegen::is_append_only_evolution;

#[test]
fn appended_field_is_append_only_safe() {
    assert!(is_append_only_evolution(
        "accepted:bool;job_id:uint64",
        "accepted:bool;job_id:uint64;reason:string"
    ));
}

#[test]
fn removed_field_is_not_append_only_safe() {
    assert!(!is_append_only_evolution(
        "accepted:bool;job_id:uint64;reason:string",
        "accepted:bool;job_id:uint64"
    ));
}

#[test]
fn reordered_fields_are_not_append_only_safe() {
    assert!(!is_append_only_evolution(
        "accepted:bool;job_id:uint64;reason:string",
        "job_id:uint64;accepted:bool;reason:string"
    ));
}
