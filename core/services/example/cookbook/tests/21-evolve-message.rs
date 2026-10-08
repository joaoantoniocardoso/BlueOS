//! Question 21: how do I evolve a message safely?
//!
//! The answer is the workflow below (append, regenerate, let pre-push check) and the three tests, which show what
//! the gate accepts and refuses. Unlike the other entries this one builds no Service: the rule is about the
//! `.msg` file, so it needs no Domain or Harness. The append-only rule (D-06) keeps older readers decoding newer
//! messages, because a field they do not know sits after every field they do.
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

// Removing a field shifts the bytes of every later one, so existing readers would decode wrongly.
#[test]
fn removed_field_is_not_append_only_safe() {
    assert!(!is_append_only_evolution(
        "accepted:bool;job_id:uint64;reason:string",
        "accepted:bool;job_id:uint64"
    ));
}

// Same fields in a new order: this is why the gate compares order, not just the set of fields.
#[test]
fn reordered_fields_are_not_append_only_safe() {
    assert!(!is_append_only_evolution(
        "accepted:bool;job_id:uint64;reason:string",
        "job_id:uint64;accepted:bool;reason:string"
    ));
}
