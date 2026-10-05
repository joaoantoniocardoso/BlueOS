//! Writes formatted generated Rust message modules.

mod format;

use std::{fs, path::Path};

use alloc::collections::BTreeMap;

use convert_case::{Case, Casing};

use crate::collect::{InterfaceRecord, MessageRecord};
use crate::error::CodegenError;
use crate::field_codec::generate_struct_tokens;
use crate::msg_ast::{DataType, FieldCase, Message};
use crate::schema::schema_text;

use format::write_formatted_rust_file;

pub(crate) fn write_rust_messages(
    records: &BTreeMap<String, MessageRecord>,
    interfaces: &[InterfaceRecord],
    out_dir: &Path,
) -> Result<(), CodegenError> {
    let packages = group_records_by_package(records);
    let package_mods = write_package_modules(&packages, records, out_dir)?;
    write_message_tree_root(out_dir, &package_mods)?;
    write_schema_match_module(out_dir, records, interfaces)?;
    Ok(())
}

fn group_records_by_package(
    records: &BTreeMap<String, MessageRecord>,
) -> BTreeMap<&str, Vec<&MessageRecord>> {
    let mut packages: BTreeMap<&str, Vec<&MessageRecord>> = BTreeMap::new();
    for record in records.values() {
        packages
            .entry(record.package.as_str())
            .or_default()
            .push(record);
    }
    packages
}

fn write_package_modules(
    packages: &BTreeMap<&str, Vec<&MessageRecord>>,
    records: &BTreeMap<String, MessageRecord>,
    out_dir: &Path,
) -> Result<Vec<String>, CodegenError> {
    let mut package_mods = Vec::new();
    for (package, package_records) in packages {
        write_one_package(out_dir, package, package_records, records)?;
        package_mods.push(format!("pub mod {package};"));
    }
    Ok(package_mods)
}

fn write_one_package(
    out_dir: &Path,
    package: &str,
    package_records: &[&MessageRecord],
    records: &BTreeMap<String, MessageRecord>,
) -> Result<(), CodegenError> {
    let package_dir = out_dir.join("msg").join(package);
    fs::create_dir_all(&package_dir)
        .map_err(|io_error| CodegenError::io(&package_dir, io_error))?;
    let message_mods = write_package_message_files(&package_dir, package_records, records)?;
    let package_mod = format!(
        "// @generated\n#![expect(\n    clippy::pub_use,\n    missing_docs,\n    reason = \"generated from ROS .msg sources\",\n)]\n{}\n",
        message_mods.join("\n")
    );
    write_formatted_rust_file(&package_dir.join("mod.rs"), &package_mod)?;
    Ok(())
}

fn write_package_message_files(
    package_dir: &Path,
    package_records: &[&MessageRecord],
    records: &BTreeMap<String, MessageRecord>,
) -> Result<Vec<String>, CodegenError> {
    let mut message_mods = Vec::new();
    for record in package_records {
        let module_name = record.name.to_case(Case::Snake);
        let file_contents = generated_message_file(record, records)?;
        write_formatted_rust_file(
            &package_dir.join(format!("{module_name}.rs")),
            &file_contents,
        )?;
        message_mods.push(format!("pub mod {module_name};"));
        message_mods.push(format!("pub use {module_name}::*;"));
    }
    Ok(message_mods)
}

fn generated_message_file(
    record: &MessageRecord,
    records: &BTreeMap<String, MessageRecord>,
) -> Result<String, CodegenError> {
    let tokens = generate_struct_tokens(record, records)?;
    let alloc_imports = alloc_imports_for_message(record);
    Ok(format!(
        "// @generated\n#![expect(\n    missing_docs,\n    reason = \"generated from ROS .msg sources\",\n)]\n{alloc_imports}use serde::{{Deserialize, Serialize}};\n\nuse crate::{{cdr, error::Error, message::{{CdrStruct, Message}}}};\n\n{tokens}\n",
        alloc_imports = alloc_imports,
        tokens = tokens
    ))
}

fn alloc_imports_for_message(record: &MessageRecord) -> String {
    let paths = alloc_paths_for_message(&record.message);
    paths
        .first()
        .map(|_| format!("use alloc::{{{}}};\n\n", paths.join(", ")))
        .unwrap_or_default()
}

fn alloc_paths_for_message(message: &Message) -> Vec<&'static str> {
    let uses_string = message
        .fields()
        .iter()
        .any(|field| matches!(field.datatype(), DataType::String));
    let uses_vec = message
        .fields()
        .iter()
        .any(|field| matches!(field.case(), FieldCase::Vector | FieldCase::Array(_)));
    let mut alloc_paths = Vec::new();
    if uses_string {
        alloc_paths.push("string::String");
    }
    if uses_vec {
        alloc_paths.push("vec::Vec");
    }
    alloc_paths
}

fn write_message_tree_root(out_dir: &Path, package_mods: &[String]) -> Result<(), CodegenError> {
    let root_mod = format!(
        "// @generated\n#![expect(\n    missing_docs,\n    reason = \"generated from ROS .msg sources\",\n)]\n{}\n",
        package_mods.join("\n")
    );
    fs::create_dir_all(out_dir.join("msg"))
        .map_err(|io_error| CodegenError::io(out_dir.join("msg"), io_error))?;
    write_formatted_rust_file(&out_dir.join("msg/mod.rs"), &root_mod)?;
    Ok(())
}

fn write_schema_match_module(
    out_dir: &Path,
    records: &BTreeMap<String, MessageRecord>,
    interfaces: &[InterfaceRecord],
) -> Result<(), CodegenError> {
    let schema_arms = schema_match_arms(records, interfaces)?;
    write_formatted_rust_file(
        &out_dir.join("mod.rs"),
        &format!(
            "// @generated\n#![expect(\n    clippy::arbitrary_source_item_ordering,\n    reason = \"generated from ROS .msg sources\"\n)]\n\npub mod msg;\n\nuse crate::message::Message;\n\n/// ROS 2 `.msg` text of any IDL message, keyed by its `SCHEMA_NAME`.\npub fn schema(schema_name: &str) -> Option<&'static str> {{\n    match schema_name {{\n{}\n        _ => None,\n    }}\n}}\n",
            schema_arms.join("\n")
        ),
    )?;
    Ok(())
}

fn schema_match_arms(
    records: &BTreeMap<String, MessageRecord>,
    interfaces: &[InterfaceRecord],
) -> Result<Vec<String>, CodegenError> {
    let mut schema_arms: Vec<String> = records
        .values()
        .map(|record| {
            format!(
                "        \"{}\" => Some(msg::{}::{}::SCHEMA),",
                record.schema_name, record.package, record.name
            )
        })
        .collect();
    for interface in interfaces {
        let text = schema_text(
            &interface.schema_name,
            &interface.source,
            &interface.dependencies,
            records,
        )?;
        schema_arms.push(format!(
            "        {:?} => Some({:?}),",
            interface.schema_name, text
        ));
    }
    Ok(schema_arms)
}
