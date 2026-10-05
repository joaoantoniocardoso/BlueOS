//! Generated CDR codec dispatch for integration tests.

use std::{fs, path::Path};

use alloc::collections::BTreeMap;

use crate::{collect::MessageRecord, error::CodegenError};

pub(crate) fn write_cdr_codec_dispatch(
    records: &BTreeMap<String, MessageRecord>,
    out_dir: &Path,
) -> Result<(), CodegenError> {
    let mut encode_arms = Vec::new();
    let mut decode_arms = Vec::new();
    for record in records.values() {
        let type_path = format!("blueos_idl::msg::{}::{}", record.package, record.name);
        encode_arms.push(format!(
            "        \"{}\" => {}::default().encode().ok(),",
            record.schema_name, type_path
        ));
        decode_arms.push(format!(
            "        \"{}\" => {}::decode(payload).ok().and_then(|message| serde_json::to_value(message).ok()),",
            record.schema_name, type_path
        ));
    }
    let contents = format!(
        "// @generated\nuse blueos_idl::Message;\n\npub(crate) fn encode_default(schema_name: &str) -> Option<Vec<u8>> {{\n    match schema_name {{\n{}\n        _ => None,\n    }}\n}}\n\npub(crate) fn decode_to_json(schema_name: &str, payload: &[u8]) -> Option<serde_json::Value> {{\n    match schema_name {{\n{}\n        _ => None,\n    }}\n}}\n",
        encode_arms.join("\n"),
        decode_arms.join("\n")
    );
    let path = out_dir.join("cdr_codec_dispatch.rs");
    fs::write(&path, contents).map_err(|source| CodegenError::io(path, source))?;
    Ok(())
}
