//! Generates committed `blueos-idl` Rust from ROS 2 `.msg` sources (`roslibrust_codegen` + `prettyplease`).

#![expect(
    clippy::pub_use,
    reason = "crate root re-exports the committed codegen API"
)]

extern crate alloc;

mod catalog;
mod cdr_dispatch;
mod cli;
mod collect;
mod constant_family;
pub mod endpoints;
mod error;
mod field_codec;
mod lock;
mod msg_ast;
mod paths;
mod rust_tokens;
mod rust_write;
mod schema;
mod typescript;

use std::{
    fs,
    path::{Path, PathBuf},
};

pub use cli::{flag_value, run as run_cli};
pub use collect::{MessageRecord, collect_messages_for_test, message_schema_names};
pub use error::CodegenError;
pub use lock::{
    check_message_lock, explain_endpoint_lock_mismatch, explain_lock_mismatch,
    field_signature_hash, format_lock_line, frozen_message_schemas, is_append_only_evolution,
    parse_lock_line,
};
pub use paths::{core_dir, idl_root};
pub use typescript::{
    message_by_schema_declares_type, missing_message_by_schema_entries, write_typescript_index,
};

pub use catalog::{
    generate_catalog, generate_catalog_outputs, generate_catalog_typescript,
    generate_schema_catalog,
};

/// Regenerates committed Rust types and schema lookup under `out_dir` (typically `blueos-idl/src/generated`).
pub fn generate(
    interfaces_root: &Path,
    out_dir: &Path,
    typescript_dir: Option<&Path>,
    test_generated_dir: Option<&Path>,
) -> Result<(), CodegenError> {
    let (messages, interfaces) = collect::collect_interfaces(interfaces_root)?;
    let records: alloc::collections::BTreeMap<String, MessageRecord> = messages
        .into_iter()
        .map(|record| (record.schema_name.clone(), record))
        .collect();

    if out_dir.exists() {
        fs::remove_dir_all(out_dir).map_err(|source| CodegenError::io(out_dir, source))?;
    }
    fs::create_dir_all(out_dir).map_err(|source| CodegenError::io(out_dir, source))?;
    let rustfmt_path = out_dir.join("rustfmt.toml");
    fs::write(&rustfmt_path, "reorder_imports = false\n")
        .map_err(|source| CodegenError::io(rustfmt_path.clone(), source))?;

    rust_write::write_rust_messages(&records, &interfaces, out_dir)?;
    let idl_root = interfaces_root
        .parent()
        .ok_or_else(|| CodegenError::MissingParent {
            path: interfaces_root.to_path_buf(),
        })?;
    let test_generated = test_generated_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| idl_root.join("tests/generated"));
    fs::create_dir_all(&test_generated)
        .map_err(|source| CodegenError::io(&test_generated, source))?;
    cdr_dispatch::write_cdr_codec_dispatch(&records, &test_generated)?;

    if let Some(typescript_dir) = typescript_dir {
        fs::create_dir_all(typescript_dir)
            .map_err(|source| CodegenError::io(typescript_dir, source))?;
        typescript::write_typescript(&records, typescript_dir)?;
    }
    Ok(())
}
