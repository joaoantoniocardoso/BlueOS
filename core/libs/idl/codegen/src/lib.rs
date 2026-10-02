//! Generates committed `blueos-idl` Rust from ROS 2 `.msg` sources (`roslibrust_codegen` + `prettyplease`).

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use roslibrust_codegen::find_and_parse_ros_messages;
use sha2::{Digest, Sha256};
mod msg_ast;

use msg_ast::{DataType, Field, FieldCase, Message};

const SCHEMA_SEPARATOR: &str =
    "================================================================================\n";

#[derive(Clone, Debug)]
pub struct MessageRecord {
    pub schema_name: String,
    pub dynamic_key: String,
    pub package: String,
    pub name: String,
    pub source_path: PathBuf,
    pub source: String,
    pub field_signature: String,
    pub type_hash: String,
    pub dependencies: BTreeSet<String>,
    pub message: Message,
}

/// Regenerates committed Rust types and schema lookup under `out_dir` (typically `blueos-idl/src/generated`).
pub fn generate(interfaces_root: &Path, out_dir: &Path) {
    let messages = collect_messages(interfaces_root);
    let records: BTreeMap<String, MessageRecord> = messages
        .into_iter()
        .map(|record| (record.schema_name.clone(), record))
        .collect();

    if out_dir.exists() {
        fs::remove_dir_all(out_dir).expect("remove stale generated dir");
    }
    fs::create_dir_all(out_dir).expect("create generated dir");

    write_rust_messages(&records, out_dir);
    let idl_root = interfaces_root
        .parent()
        .expect("interfaces directory has a parent");
    let test_generated = idl_root.join("tests/generated");
    fs::create_dir_all(&test_generated).expect("create test generated dir");
    write_cdr_codec_dispatch(&records, &test_generated);
}

/// Writes `schema_catalog.rs`, a `schema` lookup of the text of every message under `interfaces_root`, with no
/// message types, for third-party definitions that are not part of the BlueOS API.
pub fn generate_schema_catalog(interfaces_root: &Path, out_dir: &Path) {
    let records: BTreeMap<String, MessageRecord> = collect_messages(interfaces_root)
        .into_iter()
        .map(|record| (record.schema_name.clone(), record))
        .collect();
    let schema_arms: Vec<String> = records
        .values()
        .map(|record| {
            format!(
                "        {:?} => Some({:?}),",
                record.schema_name,
                schema_text(record, &records)
            )
        })
        .collect();
    fs::write(
        out_dir.join("schema_catalog.rs"),
        format!(
            "// @generated\npub fn schema(schema_name: &str) -> Option<&'static str> {{\n    match schema_name {{\n{}\n        _ => None,\n    }}\n}}\n",
            schema_arms.join("\n")
        ),
    )
    .expect("write schema catalog");
}

/// Writes `catalog.ts` with schema text for every vendored catalog message (ROS 2 and Foxglove).
pub fn generate_catalog_typescript(interfaces_root: &Path, typescript_dir: &Path) {
    let records: BTreeMap<String, MessageRecord> = collect_messages(interfaces_root)
        .into_iter()
        .map(|record| (record.schema_name.clone(), record))
        .collect();
    write_catalog_typescript(&records, typescript_dir);
}

pub fn collect_messages_for_test(interfaces_root: &Path) -> Vec<MessageRecord> {
    collect_messages(interfaces_root)
}

fn collect_messages(interfaces_root: &Path) -> Vec<MessageRecord> {
    let (parsed_messages, _, _) = find_and_parse_ros_messages(&[interfaces_root.to_path_buf()])
        .expect("parse interfaces with roslibrust_codegen");
    let mut messages = Vec::new();
    for parsed in parsed_messages {
        let package = parsed.package.clone();
        let name = parsed.name.clone();
        let dynamic_key = format!("{package}/{name}");
        let schema_name = format!("{package}/msg/{name}");
        let message = Message::from_ros_fields(&parsed.fields, &parsed.constants);
        let field_signature = msg_ast::field_signature(&message);
        let type_hash = hash_hex(&format!("{schema_name}\n{field_signature}"));
        let dependencies = message.dependencies();
        messages.push(MessageRecord {
            schema_name,
            dynamic_key,
            package,
            name,
            source_path: parsed.path.clone(),
            source: parsed.source.clone(),
            field_signature,
            type_hash,
            dependencies,
            message,
        });
    }
    messages.sort_by(|left, right| left.schema_name.cmp(&right.schema_name));
    messages
}

fn hash_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    format!("{:x}", digest)
}

fn rust_field_name(field: &Field) -> String {
    field.name().to_string()
}

fn schema_text(record: &MessageRecord, records: &BTreeMap<String, MessageRecord>) -> String {
    let mut ordered = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    fn visit(
        schema_name: &str,
        records: &BTreeMap<String, MessageRecord>,
        ordered: &mut Vec<String>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) {
        if visited.contains(schema_name) {
            return;
        }
        if visiting.contains(schema_name) {
            panic!("cycle in message dependencies at {schema_name}");
        }
        visiting.insert(schema_name.to_string());
        if let Some(record) = records.get(schema_name) {
            for dependency in &record.dependencies {
                visit(dependency, records, ordered, visiting, visited);
            }
        }
        visiting.remove(schema_name);
        if !visited.contains(schema_name) {
            ordered.push(schema_name.to_string());
            visited.insert(schema_name.to_string());
        }
    }
    for dependency in &record.dependencies {
        visit(
            dependency,
            records,
            &mut ordered,
            &mut visiting,
            &mut visited,
        );
    }
    visit(
        &record.schema_name,
        records,
        &mut ordered,
        &mut visiting,
        &mut visited,
    );

    // ROS 2 message definition layout (as in MCAP `ros2msg` schemas): the root definition first, then each
    // dependency under `MSG: package/Name`, the form `@foxglove/rosmsg` resolves (it rejects `package/msg/Name`).
    let mut blocks = vec![record.source.trim_end().to_string()];
    for schema_name in ordered {
        let Some(message_record) = records.get(&schema_name) else {
            continue;
        };
        if schema_name == record.schema_name {
            continue;
        }
        blocks.push(format!(
            "MSG: {}/{}\n{}",
            message_record.package,
            message_record.name,
            message_record.source.trim_end()
        ));
    }
    blocks.join(&format!("\n{SCHEMA_SEPARATOR}"))
}

fn write_rust_messages(records: &BTreeMap<String, MessageRecord>, out_dir: &Path) {
    let mut packages: BTreeMap<String, Vec<&MessageRecord>> = BTreeMap::new();
    for record in records.values() {
        packages
            .entry(record.package.clone())
            .or_default()
            .push(record);
    }

    let mut package_mods = Vec::new();
    for (package, package_records) in &packages {
        let package_dir = out_dir.join("msg").join(package);
        fs::create_dir_all(&package_dir).expect("create package dir");
        let mut message_mods = Vec::new();
        for record in package_records {
            let module_name = record.name.to_case(Case::Snake);
            let tokens = generate_struct_tokens(record, records);
            let uses_string = record
                .message
                .fields()
                .iter()
                .any(|field| matches!(field.datatype(), DataType::String));
            let uses_vec = record
                .message
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
            let alloc_imports = if alloc_paths.is_empty() {
                String::new()
            } else {
                format!("use alloc::{{{}}};\n\n", alloc_paths.join(", "))
            };
            let file_contents = format!(
                "// @generated\n#![allow(\n    missing_docs,\n    reason = \"generated from ROS .msg sources\",\n)]\n{alloc_imports}use crate::{{cdr, error::Error, message::{{CdrStruct, Message}}}};\nuse serde::{{Deserialize, Serialize}};\n{}\n",
                tokens
            );
            write_formatted_rust_file(
                &package_dir.join(format!("{module_name}.rs")),
                &file_contents,
            );
            message_mods.push(format!("pub mod {module_name};"));
            message_mods.push(format!("pub use {module_name}::*;"));
        }
        let package_mod = format!(
            "// @generated\n#![allow(\n    clippy::pub_use,\n    missing_docs,\n    reason = \"generated from ROS .msg sources\",\n)]\n{}\n",
            message_mods.join("\n")
        );
        write_formatted_rust_file(&package_dir.join("mod.rs"), &package_mod);
        package_mods.push(format!("pub mod {package};"));
    }

    let root_mod = format!(
        "// @generated\n#![allow(\n    missing_docs,\n    reason = \"generated from ROS .msg sources\",\n)]\n{}\n",
        package_mods.join("\n")
    );
    fs::create_dir_all(out_dir.join("msg")).expect("create msg dir");
    write_formatted_rust_file(&out_dir.join("msg/mod.rs"), &root_mod);
    let schema_arms: Vec<String> = records
        .values()
        .map(|record| {
            format!(
                "        \"{}\" => Some(msg::{}::{}::SCHEMA),",
                record.schema_name, record.package, record.name
            )
        })
        .collect();
    write_formatted_rust_file(
        &out_dir.join("mod.rs"),
        &format!(
            "// @generated\n#![allow(\n    clippy::arbitrary_source_item_ordering,\n    missing_docs,\n    reason = \"generated from ROS .msg sources\"\n)]\n\npub mod msg;\n\nuse crate::message::Message;\n\n/// ROS 2 `.msg` text of any IDL message, keyed by its `SCHEMA_NAME`.\npub fn schema(schema_name: &str) -> Option<&'static str> {{\n    match schema_name {{\n{}\n        _ => None,\n    }}\n}}\n",
            schema_arms.join("\n")
        ),
    );
}

fn write_formatted_rust_file(path: &Path, source: &str) {
    let syntax = syn::parse_file(source).expect("generated Rust must parse");
    let formatted = prettyplease::unparse(&syntax);
    fs::write(path, formatted).expect("write generated Rust");
    let status = std::process::Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg(path)
        .status()
        .expect("run rustfmt on generated Rust");
    if !status.success() {
        panic!("rustfmt failed for {}", path.display());
    }
}

fn generate_struct_tokens(
    record: &MessageRecord,
    records: &BTreeMap<String, MessageRecord>,
) -> TokenStream {
    let message = &record.message;
    let struct_name = format_ident!("{}", record.name);
    let schema_name = record.schema_name.as_str();
    let schema_text = schema_text(record, records);
    let type_hash = record.type_hash.as_str();

    let mut fields = Vec::new();
    let mut constants_mod = None;
    let mut constants = Vec::new();
    for constant in message.constants() {
        let const_name = format_ident!("{}", constant.name);
        let const_tokens = const_value_tokens(&constant.value, &constant.datatype);
        constants.push(quote! {
            pub const #const_name: #const_tokens;
        });
    }
    for field in message.fields() {
        let field_name = format_ident!("{}", rust_field_name(field));
        let field_type = rust_type_tokens(field);
        let serde_with = if matches!(field.case(), FieldCase::Array(_)) {
            quote! { #[serde(with = "serde_arrays")] }
        } else {
            quote! {}
        };
        fields.push(quote! {
            #serde_with
            pub #field_name: #field_type,
        });
    }
    if !constants.is_empty() {
        let constants_name = format_ident!("constants_{}", record.name.to_case(Case::Snake));
        constants_mod = Some(quote! {
            pub mod #constants_name {
                #(#constants)*
            }
        });
    }

    let encode_fields = encode_field_tokens(message);
    let decode_tokens = decode_field_tokens(message);
    let decode_assignments = decode_tokens.assignments;
    let field_count = message.fields().len();

    quote! {
        #constants_mod
        #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
        pub struct #struct_name {
            #(#fields)*
        }

        impl CdrStruct for #struct_name {
            fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
                Ok(Self {
                    #decode_assignments
                })
            }

            fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
                #(#encode_fields)*
                Ok(())
            }
        }

        impl Message for #struct_name {
            const SCHEMA: &'static str = #schema_text;
            const SCHEMA_NAME: &'static str = #schema_name;
            const TYPE_HASH: &'static str = #type_hash;
        }

        impl #struct_name {
            pub const KNOWN_FIELD_COUNT: usize = #field_count;
        }
    }
}

struct DecodeFieldTokens {
    assignments: TokenStream,
}

fn decode_field_tokens(message: &Message) -> DecodeFieldTokens {
    let mut assignments = Vec::new();
    for field in message.fields() {
        let field_name = format_ident!("{}", rust_field_name(field));
        let read = read_field_tokens(field, &field_name);
        assignments.push(quote! {
            #field_name: #read,
        });
    }
    DecodeFieldTokens {
        assignments: quote! { #(#assignments)* },
    }
}

fn read_field_tokens(field: &Field, _field_name: &proc_macro2::Ident) -> TokenStream {
    match field.case() {
        FieldCase::Vector => {
            let element = read_scalar_or_message_inner(field);
            quote! {
                {
                    if reader.is_exhausted() {
                        Vec::new()
                    } else {
                        let length = reader.read_bounded_sequence_length()?;
                        let mut values = Vec::with_capacity(length as usize);
                        for _index in 0..length {
                            values.push(#element);
                        }
                        values
                    }
                }
            }
        }
        FieldCase::Array(size) => {
            let element = read_scalar_or_message_inner(field);
            quote! {
                {
                    let mut values = [Default::default(); #size];
                    if !reader.is_exhausted() {
                        for index in 0..#size {
                            values[index] = #element;
                        }
                    }
                    values
                }
            }
        }
        _ => read_scalar_or_message(field, false),
    }
}

fn read_scalar_or_message(field: &Field, _nested: bool) -> TokenStream {
    let default_value = default_for_field(field);
    let read = read_scalar_or_message_inner(field);
    quote! {
        if reader.is_exhausted() {
            #default_value
        } else {
            #read
        }
    }
}

fn read_scalar_or_message_inner(field: &Field) -> TokenStream {
    match field.datatype() {
        DataType::String => quote! { reader.read_string()? },
        DataType::Bool => quote! { reader.read_bool()? },
        DataType::U8 => quote! { reader.read_u8()? },
        DataType::U16 => quote! { reader.read_u16()? },
        DataType::U32 => quote! { reader.read_u32()? },
        DataType::U64 => quote! { reader.read_u64()? },
        DataType::I8 => quote! { reader.read_i8()? },
        DataType::I16 => quote! { reader.read_i16()? },
        DataType::I32 => quote! { reader.read_i32()? },
        DataType::I64 => quote! { reader.read_i64()? },
        DataType::F32 => quote! { reader.read_f32()? },
        DataType::F64 => quote! { reader.read_f64()? },
        DataType::GlobalMessage { package, name } => {
            let package = format_ident!("{}", package);
            let name = format_ident!("{}", name);
            quote! { <crate::msg::#package::#name>::cdr_decode_fields(reader)? }
        }
    }
}

fn encode_field_tokens(message: &Message) -> Vec<TokenStream> {
    let mut tokens = Vec::new();
    for field in message.fields() {
        let field_name = format_ident!("{}", rust_field_name(field));
        tokens.push(write_field_tokens(field, quote! { self.#field_name }));
    }
    tokens
}

fn write_field_tokens(field: &Field, value: TokenStream) -> TokenStream {
    match field.case() {
        FieldCase::Vector => {
            let element_write = write_vector_element(field);
            quote! {
                writer.write_u32(#value.len() as u32)?;
                for element in #value.iter() {
                    #element_write
                }
            }
        }
        FieldCase::Array(_) => {
            let element_write = write_vector_element(field);
            quote! {
                for element in #value.iter() {
                    #element_write
                }
            }
        }
        FieldCase::Scalar | FieldCase::Const(_) => write_scalar_or_message(field, value),
    }
}

fn write_vector_element(field: &Field) -> TokenStream {
    match field.datatype() {
        DataType::GlobalMessage { package, name } => {
            let package = format_ident!("{}", package);
            let name = format_ident!("{}", name);
            quote! {
                <crate::msg::#package::#name>::cdr_encode_fields(element, writer)?;
            }
        }
        DataType::String => quote! { writer.write_string(element.as_str())?; },
        _ => write_scalar_or_message(field, quote! { *element }),
    }
}

fn write_scalar_or_message(field: &Field, value: TokenStream) -> TokenStream {
    match field.datatype() {
        DataType::String => quote! { writer.write_string(#value.as_str())?; },
        DataType::Bool => quote! { writer.write_bool(#value)?; },
        DataType::U8 => quote! { writer.write_u8(#value)?; },
        DataType::U16 => quote! { writer.write_u16(#value)?; },
        DataType::U32 => quote! { writer.write_u32(#value)?; },
        DataType::U64 => quote! { writer.write_u64(#value)?; },
        DataType::I8 => quote! { writer.write_i8(#value)?; },
        DataType::I16 => quote! { writer.write_i16(#value)?; },
        DataType::I32 => quote! { writer.write_i32(#value)?; },
        DataType::I64 => quote! { writer.write_i64(#value)?; },
        DataType::F32 => quote! { writer.write_f32(#value)?; },
        DataType::F64 => quote! { writer.write_f64(#value)?; },
        DataType::GlobalMessage { package, name } => {
            let package = format_ident!("{}", package);
            let name = format_ident!("{}", name);
            quote! { <crate::msg::#package::#name>::cdr_encode_fields(&#value, writer)?; }
        }
    }
}

fn default_for_field(field: &Field) -> TokenStream {
    match field.case() {
        FieldCase::Vector => quote! { Vec::new() },
        FieldCase::Array(size) => {
            let element_default = match field.datatype() {
                DataType::GlobalMessage { package, name } => {
                    let package = format_ident!("{}", package);
                    let name = format_ident!("{}", name);
                    quote! { <crate::msg::#package::#name>::default() }
                }
                _ => quote! { Default::default() },
            };
            quote! { [#element_default; #size] }
        }
        FieldCase::Scalar | FieldCase::Const(_) => match field.datatype() {
            DataType::String => quote! { String::new() },
            DataType::Bool => quote! { false },
            DataType::GlobalMessage { package, name } => {
                let package = format_ident!("{}", package);
                let name = format_ident!("{}", name);
                quote! { <crate::msg::#package::#name>::default() }
            }
            _ => quote! { Default::default() },
        },
    }
}

fn rust_type_tokens(field: &Field) -> TokenStream {
    let base = match field.datatype() {
        DataType::String => quote! { String },
        DataType::Bool => quote! { bool },
        DataType::U8 => quote! { u8 },
        DataType::U16 => quote! { u16 },
        DataType::U32 => quote! { u32 },
        DataType::U64 => quote! { u64 },
        DataType::I8 => quote! { i8 },
        DataType::I16 => quote! { i16 },
        DataType::I32 => quote! { i32 },
        DataType::I64 => quote! { i64 },
        DataType::F32 => quote! { f32 },
        DataType::F64 => quote! { f64 },
        DataType::GlobalMessage { package, name } => {
            let package = format_ident!("{}", package);
            let name = format_ident!("{}", name);
            quote! { crate::msg::#package::#name }
        }
    };
    match field.case() {
        FieldCase::Vector => quote! { Vec<#base> },
        FieldCase::Array(size) => quote! { [#base; #size] },
        FieldCase::Scalar | FieldCase::Const(_) => base,
    }
}

fn const_value_tokens(value: &msg_ast::ConstantValue, datatype: &DataType) -> TokenStream {
    use msg_ast::ConstantValue;
    match (datatype, value) {
        (DataType::U8, ConstantValue::U8(value)) => quote! { u8 = #value },
        (DataType::U16, ConstantValue::U16(value)) => quote! { u16 = #value },
        (DataType::U32, ConstantValue::U32(value)) => quote! { u32 = #value },
        (DataType::U64, ConstantValue::U64(value)) => quote! { u64 = #value },
        (DataType::I8, ConstantValue::I8(value)) => quote! { i8 = #value },
        (DataType::I16, ConstantValue::I16(value)) => quote! { i16 = #value },
        (DataType::I32, ConstantValue::I32(value)) => quote! { i32 = #value },
        (DataType::I64, ConstantValue::I64(value)) => quote! { i64 = #value },
        (DataType::F32, ConstantValue::F32(value)) => quote! { f32 = #value },
        (DataType::F64, ConstantValue::F64(value)) => quote! { f64 = #value },
        (DataType::String, ConstantValue::String(value)) => quote! { &'static str = #value },
        _ => quote! { () },
    }
}

fn write_catalog_typescript(records: &BTreeMap<String, MessageRecord>, typescript_dir: &Path) {
    let mut schema_entries = Vec::new();
    for record in records.values() {
        let schema = schema_text(record, records);
        schema_entries.push(format!(
            "  \"{}\": `{}`,",
            record.schema_name,
            schema.replace('`', "\\`")
        ));
    }
    fs::write(
        typescript_dir.join("catalog.ts"),
        format!(
            "// @generated\nexport const CATALOG_SCHEMAS: Record<string, string> = {{\n{}\n}};\n",
            schema_entries.join("\n")
        ),
    )
    .expect("write catalog.ts");
}

pub fn field_signature_hash(field_signature: &str) -> String {
    hash_hex(field_signature)
}

pub fn is_append_only_evolution(previous: &str, current: &str) -> bool {
    if previous == current {
        return true;
    }
    if current.len() <= previous.len() {
        return false;
    }
    if !current.starts_with(previous) {
        return false;
    }
    let suffix = current.strip_prefix(previous).unwrap_or("");
    suffix.is_empty() || suffix.starts_with(';')
}

pub fn parse_lock_line(line: &str) -> Option<(String, u32, String)> {
    let mut parts = line.split_whitespace();
    let schema_name = parts.next()?.to_string();
    let major = parts.next()?.parse().ok()?;
    let hash = parts.next()?.to_string();
    Some((schema_name, major, hash))
}

pub fn format_lock_line(schema_name: &str, major: u32, field_signature: &str) -> String {
    format!(
        "{} {} {}",
        schema_name,
        major,
        field_signature_hash(field_signature)
    )
}

fn write_cdr_codec_dispatch(records: &BTreeMap<String, MessageRecord>, out_dir: &Path) {
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
    fs::write(
        out_dir.join("cdr_codec_dispatch.rs"),
        format!(
            "// @generated\nuse blueos_idl::Message;\n\npub fn encode_default(schema_name: &str) -> Option<Vec<u8>> {{\n    match schema_name {{\n{}\n        _ => None,\n    }}\n}}\n\npub fn decode_to_json(schema_name: &str, payload: &[u8]) -> Option<serde_json::Value> {{\n    match schema_name {{\n{}\n        _ => None,\n    }}\n}}\n",
            encode_arms.join("\n"),
            decode_arms.join("\n")
        ),
    )
    .expect("write cdr_codec_dispatch.rs");
}
