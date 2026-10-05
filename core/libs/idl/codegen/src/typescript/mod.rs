//! TypeScript message and schema emission.

mod emit;

use std::{fs, path::Path};

use alloc::collections::BTreeMap;

use convert_case::{Case, Casing};

use crate::collect::MessageRecord;
use crate::constant_family::ConstantFamily;
use crate::error::CodegenError;
use crate::msg_ast::{ConstantValue, DataType, Field, FieldCase};
use crate::rust_tokens::enum_variant_ident;
use crate::schema::schema_text;

pub(crate) fn write_typescript(
    records: &BTreeMap<String, MessageRecord>,
    typescript_dir: &Path,
) -> Result<(), CodegenError> {
    let artifacts = emit::collect_typescript_artifacts(records)?;
    let messages_source = format!(
        "// @generated\n\n{}\nexport interface MessageBySchema {{\n{}\n}}\n",
        artifacts.interface_blocks, artifacts.message_by_schema
    );
    let schemas_source = format!(
        "// @generated\nexport const SCHEMAS: Record<string, string> = {{\n{}\n}};\n",
        artifacts.schemas_source
    );
    let missing = missing_message_by_schema_entries(&schemas_source, &messages_source);
    if !missing.is_empty() {
        return Err(CodegenError::Cli(format!(
            "MessageBySchema is missing schema entries: {missing:?}"
        )));
    }
    write_typescript_files(
        typescript_dir,
        &artifacts,
        &messages_source,
        &schemas_source,
    )?;
    write_typescript_index(typescript_dir)
}

fn write_typescript_files(
    typescript_dir: &Path,
    artifacts: &emit::TypeScriptArtifacts,
    messages_source: &str,
    schemas_source: &str,
) -> Result<(), CodegenError> {
    let constants_path = typescript_dir.join("constants.ts");
    fs::write(
        &constants_path,
        format!("// @generated\n\n{}", artifacts.constants_source),
    )
    .map_err(|source| CodegenError::io(constants_path, source))?;
    let messages_path = typescript_dir.join("messages.d.ts");
    fs::write(&messages_path, messages_source)
        .map_err(|source| CodegenError::io(messages_path, source))?;
    let schemas_path = typescript_dir.join("schemas.ts");
    fs::write(&schemas_path, schemas_source)
        .map_err(|source| CodegenError::io(schemas_path, source))?;
    Ok(())
}

/// Writes `index.ts` in `typescript_dir`, exporting the generated constants and schemas, and the catalog when
/// `catalog.ts` is there.
pub fn write_typescript_index(typescript_dir: &Path) -> Result<(), CodegenError> {
    let catalog_path = typescript_dir.join("catalog.ts");
    let catalog_export = if catalog_path.exists() {
        "export * from \"./catalog\";\n"
    } else {
        ""
    };
    let index_path = typescript_dir.join("index.ts");
    fs::write(
        &index_path,
        format!(
            "// @generated\nexport * from \"./constants\";\nexport * from \"./schemas\";\n{catalog_export}",
            catalog_export = catalog_export
        ),
    )
    .map_err(|source| CodegenError::io(index_path, source))?;
    Ok(())
}

pub(crate) fn write_catalog_typescript(
    records: &BTreeMap<String, MessageRecord>,
    typescript_dir: &Path,
) -> Result<(), CodegenError> {
    let mut schema_entries = Vec::new();
    for record in records.values() {
        let schema = schema_text(
            &record.schema_name,
            &record.source,
            &record.dependencies,
            records,
        )?;
        schema_entries.push(format!(
            "  \"{}\": `{}`,",
            record.schema_name,
            schema.replace('`', "\\`")
        ));
    }
    let catalog_path = typescript_dir.join("catalog.ts");
    fs::write(
        &catalog_path,
        format!(
            "// @generated\nexport const CATALOG_SCHEMAS: Record<string, string> = {{\n{}\n}};\n",
            schema_entries.join("\n")
        ),
    )
    .map_err(|source| CodegenError::io(catalog_path, source))?;
    Ok(())
}

pub(crate) fn typescript_constant_block(message_name: &str, family: &ConstantFamily) -> String {
    let export_name = format!(
        "{}{}",
        message_name,
        family.field_name.to_case(Case::Pascal)
    );
    let mut entries = Vec::new();
    for constant in &family.constants {
        let variant = enum_variant_ident(constant, family);
        let raw = typescript_constant_value(&constant.value);
        entries.push(format!("  {variant}: {raw},"));
    }
    format!(
        "export const {export_name} = {{\n{}\n}} as const;\nexport type {export_name} = typeof {export_name}[keyof typeof {export_name}] | number;\n",
        entries.join("\n")
    )
}

pub(crate) fn typescript_constant_value(value: &ConstantValue) -> String {
    match value {
        ConstantValue::String(text) => format!("\"{text}\""),
        ConstantValue::F32(value) => value.to_string(),
        ConstantValue::F64(value) => value.to_string(),
        ConstantValue::U8(value) => value.to_string(),
        ConstantValue::U16(value) => value.to_string(),
        ConstantValue::U32(value) => value.to_string(),
        ConstantValue::U64(value) => value.to_string(),
        ConstantValue::I8(value) => value.to_string(),
        ConstantValue::I16(value) => value.to_string(),
        ConstantValue::I32(value) => value.to_string(),
        ConstantValue::I64(value) => value.to_string(),
    }
}

/// Parses schema names from a generated `schemas.ts` or `catalog.ts` body.
pub(crate) fn schema_names_from_schemas_source(schemas_source: &str) -> Vec<String> {
    schemas_source
        .lines()
        .filter_map(|line| line.strip_prefix("  \"")?.split_once("\": `"))
        .map(|(schema_name, _)| schema_name.to_owned())
        .collect()
}

/// The TypeScript interface name BlueOS generates for a ROS schema name.
pub(crate) fn typescript_type_name_for_schema(schema_name: &str) -> String {
    schema_name
        .rsplit('/')
        .next()
        .unwrap_or(schema_name)
        .replace('_', "")
}

/// Schema names listed in `schemas_source` that are missing from `MessageBySchema` in `messages_dts`.
pub fn missing_message_by_schema_entries(schemas_source: &str, messages_dts: &str) -> Vec<String> {
    schema_names_from_schemas_source(schemas_source)
        .into_iter()
        .filter(|schema_name| !message_by_schema_declares_type(messages_dts, schema_name))
        .collect()
}

/// Whether `messages_dts` declares `MessageBySchema[schema_name]` with the expected type name.
pub fn message_by_schema_declares_type(messages_dts: &str, schema_name: &str) -> bool {
    let type_name = typescript_type_name_for_schema(schema_name);
    messages_dts.contains(&format!("  \"{schema_name}\": {type_name};\n"))
}

/// The TypeScript type of `field`. A `uint8` sequence is a `Uint8Array`, which is what the CDR reader returns.
pub(crate) fn typescript_type(field: &Field) -> String {
    if matches!(field.datatype(), DataType::U8)
        && matches!(field.case(), FieldCase::Vector | FieldCase::Array(_))
    {
        return "Uint8Array".to_string();
    }
    let base = match field.datatype() {
        DataType::String => "string".to_string(),
        DataType::Bool => "boolean".to_string(),
        DataType::U8
        | DataType::U16
        | DataType::U32
        | DataType::U64
        | DataType::I8
        | DataType::I16
        | DataType::I32
        | DataType::I64
        | DataType::F32
        | DataType::F64 => "number".to_string(),
        DataType::GlobalMessage { name, .. } => name,
    };
    match field.case() {
        FieldCase::Vector | FieldCase::Array(_) => format!("{base}[]"),
        FieldCase::Scalar | FieldCase::Const(_) => base,
    }
}
