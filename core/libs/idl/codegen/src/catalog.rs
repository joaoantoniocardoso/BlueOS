//! Vendored catalog schema lookup generation.

use std::{fs, path::Path};

use alloc::collections::BTreeMap;

use crate::{
    collect::{MessageRecord, collect_messages},
    error::CodegenError,
    schema::schema_text,
    typescript::{write_catalog_typescript, write_typescript_index},
};

/// Regenerates committed catalog schema lookup and TypeScript (`catalog.ts`).
pub fn generate_catalog(idl_root: &Path) -> Result<(), CodegenError> {
    let typescript_dir = idl_root.join("typescript");
    generate_catalog_outputs(idl_root, &idl_root.join("src/generated"), &typescript_dir)
}

/// Writes vendored catalog outputs next to interface codegen artifacts.
pub fn generate_catalog_outputs(
    idl_root: &Path,
    generated_dir: &Path,
    typescript_dir: &Path,
) -> Result<(), CodegenError> {
    let catalog_interfaces = idl_root.join("catalog/interfaces");
    fs::create_dir_all(generated_dir).map_err(|source| CodegenError::io(generated_dir, source))?;
    fs::create_dir_all(typescript_dir)
        .map_err(|source| CodegenError::io(typescript_dir, source))?;
    generate_schema_catalog(&catalog_interfaces, generated_dir)?;
    generate_catalog_typescript(&catalog_interfaces, typescript_dir)?;
    write_typescript_index(typescript_dir)?;
    Ok(())
}

/// Writes `schema_catalog.rs`, a `schema` lookup of the text of every message under `interfaces_root`, with no
/// message types, for third-party definitions that are not part of the BlueOS API.
pub fn generate_schema_catalog(interfaces_root: &Path, out_dir: &Path) -> Result<(), CodegenError> {
    let records: BTreeMap<String, MessageRecord> = collect_messages(interfaces_root)?
        .into_iter()
        .map(|record| (record.schema_name.clone(), record))
        .collect();
    let mut schema_arms = Vec::new();
    for record in records.values() {
        let text = schema_text(
            &record.schema_name,
            &record.source,
            &record.dependencies,
            &records,
        )?;
        schema_arms.push(format!(
            "        {:?} => Some({:?}),",
            record.schema_name, text
        ));
    }
    let path = out_dir.join("schema_catalog.rs");
    let contents = format!(
        "// @generated\npub(crate) fn schema(schema_name: &str) -> Option<&'static str> {{\n    match schema_name {{\n{}\n        _ => None,\n    }}\n}}\n",
        schema_arms.join("\n")
    );
    fs::write(&path, contents).map_err(|source| CodegenError::io(path, source))?;
    Ok(())
}

/// Writes `catalog.ts` with schema text for every vendored catalog message (ROS 2 and Foxglove).
pub fn generate_catalog_typescript(
    interfaces_root: &Path,
    typescript_dir: &Path,
) -> Result<(), CodegenError> {
    let records: BTreeMap<String, MessageRecord> = collect_messages(interfaces_root)?
        .into_iter()
        .map(|record| (record.schema_name.clone(), record))
        .collect();
    write_catalog_typescript(&records, typescript_dir)
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf};

    use super::*;

    #[test]
    fn generate_catalog_writes_committed_layout() {
        let idl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("idl root")
            .to_path_buf();
        let output = env::temp_dir().join(format!(
            "blueos-idl-catalog-{}-{}",
            env!("CARGO_PKG_NAME"),
            std::process::id()
        ));
        _ = fs::remove_dir_all(&output);
        fs::create_dir_all(&output).expect("temp dir");
        let typescript = output.join("typescript");
        generate_catalog_outputs(&idl_root, &output, &typescript).expect("catalog outputs");
        assert!(output.join("schema_catalog.rs").is_file());
        assert!(typescript.join("catalog.ts").is_file());
        generate_catalog_typescript(&idl_root.join("catalog/interfaces"), &typescript)
            .expect("catalog typescript");
        _ = fs::remove_dir_all(output);
    }
}
