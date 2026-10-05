//! Collected TypeScript artifacts before writing files.

use alloc::collections::BTreeMap;

use crate::{
    collect::MessageRecord,
    error::CodegenError,
    msg_ast::FieldCase,
    rust_tokens::constant_families_by_field,
    schema::{rust_field_name, schema_text},
};

pub(super) struct TypeScriptArtifacts {
    pub(super) constants_source: String,
    pub(super) interface_blocks: String,
    pub(super) message_by_schema: String,
    pub(super) schemas_source: String,
}

struct RecordTypeScriptPieces {
    message_by_schema: String,
    constant_blocks: Vec<String>,
    interface_block: String,
    schema_entry: String,
}

pub(super) fn collect_typescript_artifacts(
    records: &BTreeMap<String, MessageRecord>,
) -> Result<TypeScriptArtifacts, CodegenError> {
    let pieces = typescript_pieces_for_records(records)?;
    Ok(merge_typescript_pieces(pieces))
}

fn typescript_pieces_for_records(
    records: &BTreeMap<String, MessageRecord>,
) -> Result<Vec<RecordTypeScriptPieces>, CodegenError> {
    records
        .values()
        .map(|record| record_typescript_pieces(record, records))
        .collect()
}

fn merge_typescript_pieces(pieces: Vec<RecordTypeScriptPieces>) -> TypeScriptArtifacts {
    TypeScriptArtifacts {
        constants_source: typescript_constants_source(&pieces),
        interface_blocks: joined_piece_field(&pieces, |piece| piece.interface_block.as_str()),
        message_by_schema: joined_piece_field(&pieces, |piece| piece.message_by_schema.as_str()),
        schemas_source: joined_piece_field(&pieces, |piece| piece.schema_entry.as_str()),
    }
}

fn typescript_constants_source(pieces: &[RecordTypeScriptPieces]) -> String {
    let mut block = String::new();
    for constant in pieces.iter().flat_map(|piece| piece.constant_blocks.iter()) {
        block.push_str(constant);
        block.push('\n');
    }
    block
}

fn joined_piece_field(
    pieces: &[RecordTypeScriptPieces],
    field: impl Fn(&RecordTypeScriptPieces) -> &str,
) -> String {
    pieces.iter().map(field).collect::<Vec<_>>().join("\n")
}

fn record_typescript_pieces(
    record: &MessageRecord,
    records: &BTreeMap<String, MessageRecord>,
) -> Result<RecordTypeScriptPieces, CodegenError> {
    let interface_name = record.name.as_str();
    let families = constant_families_by_field(&record.message);
    let constant_blocks = families
        .values()
        .map(|family| super::typescript_constant_block(interface_name, family))
        .collect();
    let schema = schema_text(
        &record.schema_name,
        &record.source,
        &record.dependencies,
        records,
    )?;
    Ok(RecordTypeScriptPieces {
        message_by_schema: format!("  \"{}\": {interface_name};", record.schema_name),
        constant_blocks,
        interface_block: interface_block(record),
        schema_entry: format!(
            "  \"{}\": `{}`,",
            record.schema_name,
            schema.replace('`', "\\`")
        ),
    })
}

fn interface_block(record: &MessageRecord) -> String {
    let interface_name = record.name.as_str();
    let mut fields = Vec::new();
    for field in record.message.fields() {
        if matches!(field.case(), FieldCase::Const(_)) {
            continue;
        }
        let field_name = rust_field_name(field);
        let ts_name = if field_name == "type_" {
            "type".to_string()
        } else {
            field_name
        };
        let ts_type = super::typescript_type(field);
        fields.push(format!("  {ts_name}: {ts_type};"));
    }
    if fields.is_empty() {
        format!("export interface {interface_name} {{}}\n")
    } else {
        format!(
            "export interface {interface_name} {{\n{}\n}}\n",
            fields.join("\n")
        )
    }
}
