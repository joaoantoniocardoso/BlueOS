use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use ros2_message::dynamic::DynamicMsg;
use ros2_message::{DataType, FieldCase, FieldInfo};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

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
}

pub fn generate(interfaces_root: &Path, out_dir: &Path, typescript_dir: &Path) {
    let messages = collect_messages(interfaces_root);
    let records: BTreeMap<String, MessageRecord> = messages
        .into_iter()
        .map(|record| (record.schema_name.clone(), record))
        .collect();

    fs::create_dir_all(out_dir).expect("create OUT_DIR");
    fs::create_dir_all(typescript_dir).expect("create typescript dir");

    write_rust_messages(&records, out_dir);
    write_signatures(&records, out_dir);
    write_typescript(&records, typescript_dir);
}

pub fn collect_messages_for_test(interfaces_root: &Path) -> Vec<MessageRecord> {
    collect_messages(interfaces_root)
}

fn collect_messages(interfaces_root: &Path) -> Vec<MessageRecord> {
    let mut messages = Vec::new();

    for entry in WalkDir::new(interfaces_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "msg") {
            continue;
        }

        let relative = path
            .strip_prefix(interfaces_root)
            .expect("message under interfaces root");
        let parts: Vec<&str> = relative
            .components()
            .map(|component| component.as_os_str().to_str().expect("utf-8 path"))
            .collect();
        if parts.len() != 3 || parts[1] != "msg" {
            panic!("expected <package>/msg/<Name>.msg, got {relative:?}");
        }
        let package = parts[0].to_string();
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("message file name")
            .to_string();
        let dynamic_key = format!("{package}/{name}");
        let schema_name = format!("{package}/msg/{name}");
        let source = fs::read_to_string(path).expect("read message source");
        let dynamic_message = DynamicMsg::new(&dynamic_key, &parseable_source(&source))
            .expect("parse message source");
        let message = dynamic_message.msg();
        let field_signature = field_signature(message);
        let type_hash = hash_hex(&format!("{schema_name}\n{field_signature}"));
        let dependencies = dependency_schema_names(message);

        messages.push(MessageRecord {
            schema_name,
            dynamic_key,
            package,
            name,
            source_path: path.to_path_buf(),
            source,
            field_signature,
            type_hash,
            dependencies,
        });
    }

    messages.sort_by(|left, right| left.schema_name.cmp(&right.schema_name));
    messages
}

/// `ros2_message` rejects bounded strings and sequences (`string<=255`, `float64[<=3]`) and field defaults
/// (`bool read_only false`). Neither changes the CDR encoding, so they are dropped for parsing while the schema text
/// keeps them.
fn parseable_source(source: &str) -> String {
    let mut unbounded = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("<=") {
        unbounded.push_str(&rest[..start]);
        rest = rest[start + 2..].trim_start_matches(|character: char| character.is_ascii_digit());
    }
    unbounded.push_str(rest);
    unbounded
        .lines()
        .map(|line| {
            let definition = line.split('#').next().unwrap_or_default();
            if definition.contains('=') {
                line.to_string()
            } else {
                definition
                    .split_whitespace()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn dependency_schema_names(message: &ros2_message::Msg) -> BTreeSet<String> {
    message
        .dependencies()
        .iter()
        .map(|path| format!("{}/msg/{}", path.package(), path.name()))
        .collect()
}

fn field_signature(message: &ros2_message::Msg) -> String {
    let mut parts = Vec::new();
    for field in message.fields() {
        if matches!(field.case(), FieldCase::Const(_)) {
            continue;
        }
        let field_name = rust_field_name(field);
        let field_type = ros2_field_type_name(field);
        parts.push(format!("{field_name}:{field_type}"));
    }
    parts.join(";")
}

fn ros2_field_type_name(field: &FieldInfo) -> String {
    let base = match field.datatype() {
        DataType::String => "string".to_string(),
        DataType::Bool => "bool".to_string(),
        DataType::U8(_) => "uint8".to_string(),
        DataType::U16 => "uint16".to_string(),
        DataType::U32 => "uint32".to_string(),
        DataType::U64 => "uint64".to_string(),
        DataType::I8(_) => "int8".to_string(),
        DataType::I16 => "int16".to_string(),
        DataType::I32 => "int32".to_string(),
        DataType::I64 => "int64".to_string(),
        DataType::F32 => "float32".to_string(),
        DataType::F64 => "float64".to_string(),
        DataType::GlobalMessage(path) => format!("{}/{}", path.package(), path.name()),
        other => other.to_string(),
    };
    match field.case() {
        FieldCase::Vector => format!("{base}[]"),
        FieldCase::Array(size) => format!("{base}[{size}]"),
        _ => base,
    }
}

fn hash_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    format!("{:x}", digest)
}

fn rust_field_name(field: &FieldInfo) -> String {
    if field.name() == "type" {
        "type_".to_string()
    } else {
        field.name().to_string()
    }
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

fn write_signatures(records: &BTreeMap<String, MessageRecord>, out_dir: &Path) {
    let mut lines = Vec::new();
    for (schema_name, record) in records {
        lines.push(format!(
            "pub const SIGNATURE_{}: (&str, &str) = (\"{}\", \"{}\");",
            const_suffix(schema_name),
            schema_name,
            record.field_signature
        ));
    }
    let content = format!(
        "// @generated\npub const MESSAGE_SIGNATURES: &[(&str, &str)] = &[\n{}\n];\n",
        records
            .values()
            .map(|record| {
                format!(
                    "    (\"{}\", \"{}\"),",
                    record.schema_name, record.field_signature
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    );
    fs::write(out_dir.join("signatures.rs"), content).expect("write signatures");
    let _ = lines;
}

fn const_suffix(schema_name: &str) -> String {
    schema_name
        .replace('/', "_")
        .replace("msg_", "")
        .to_case(Case::UpperSnake)
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
            let file_contents = format!(
                "// @generated\n#![allow(
    clippy::derivable_impls,
    clippy::needless_borrow,
    clippy::needless_pass_by_ref_mut,
    clippy::too_many_lines,
    unused_imports,
    unused_mut,
)]\nuse crate::{{cdr, error::Error, message::{{CdrStruct, Message}}}};\nuse alloc::string::String;\nuse alloc::vec::Vec;\n{}\n",
                tokens
            );
            fs::write(package_dir.join(format!("{module_name}.rs")), file_contents)
                .expect("write message module");
            message_mods.push(format!("pub mod {module_name};"));
            message_mods.push(format!("pub use {module_name}::*;"));
        }
        let package_mod = format!("// @generated\n{}\n", message_mods.join("\n"));
        fs::write(package_dir.join("mod.rs"), package_mod).expect("write package mod");
        package_mods.push(format!("pub mod {package};"));
    }

    let root_mod = format!("// @generated\n{}\n", package_mods.join("\n"));
    fs::create_dir_all(out_dir.join("msg")).expect("create msg dir");
    fs::write(out_dir.join("msg/mod.rs"), root_mod).expect("write msg mod");
    let schema_arms: Vec<String> = records
        .values()
        .map(|record| {
            format!(
                "        \"{}\" => Some(msg::{}::{}::SCHEMA),",
                record.schema_name, record.package, record.name
            )
        })
        .collect();
    fs::write(
        out_dir.join("generated_mod.rs"),
        format!(
            "// @generated\npub mod msg;\n\nuse crate::message::Message;\n\n/// ROS 2 `.msg` text of any IDL message, keyed by its `SCHEMA_NAME`.\npub fn schema(schema_name: &str) -> Option<&'static str> {{\n    match schema_name {{\n{}\n        _ => None,\n    }}\n}}\n",
            schema_arms.join("\n")
        ),
    )
    .expect("write generated mod");
}

fn generate_struct_tokens(
    record: &MessageRecord,
    records: &BTreeMap<String, MessageRecord>,
) -> TokenStream {
    let dynamic_message =
        DynamicMsg::new(&record.dynamic_key, &record.source).expect("parse for codegen");
    let message = dynamic_message.msg();
    let struct_name = format_ident!("{}", record.name);
    let schema_name = record.schema_name.as_str();
    let schema_text = schema_text(record, records);
    let type_hash = record.type_hash.as_str();

    let mut fields = Vec::new();
    let mut constants_mod = None;
    let mut constants = Vec::new();
    for field in message.fields() {
        if matches!(field.case(), FieldCase::Const(_)) {
            if let Some(value) = field.const_value() {
                let const_name = format_ident!("{}", rust_field_name(field));
                let const_tokens = const_value_tokens(value.clone());
                constants.push(quote! {
                    pub const #const_name: #const_tokens;
                });
            }
            continue;
        }
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
    let decode_defaults = decode_tokens.defaults;
    let decode_assignments = decode_tokens.assignments;
    let field_count = message
        .fields()
        .iter()
        .filter(|field| !matches!(field.case(), FieldCase::Const(_)))
        .count();

    quote! {
        use serde::{Deserialize, Serialize};
        #constants_mod
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct #struct_name {
            #(#fields)*
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    #decode_defaults
                }
            }
        }

        impl CdrStruct for #struct_name {
            fn cdr_encode_fields(&self, mut writer: &mut cdr::Writer) -> Result<(), Error> {
                #(#encode_fields)*
                Ok(())
            }

            fn cdr_decode_fields(mut reader: &mut cdr::Reader) -> Result<Self, Error> {
                Ok(Self {
                    #decode_assignments
                })
            }
        }

        impl Message for #struct_name {
            const SCHEMA_NAME: &'static str = #schema_name;
            const SCHEMA: &'static str = #schema_text;
            const TYPE_HASH: &'static str = #type_hash;
        }

        impl #struct_name {
            pub const KNOWN_FIELD_COUNT: usize = #field_count;
        }
    }
}

struct DecodeFieldTokens {
    defaults: TokenStream,
    assignments: TokenStream,
}

fn decode_field_tokens(message: &ros2_message::Msg) -> DecodeFieldTokens {
    let mut defaults = Vec::new();
    let mut assignments = Vec::new();
    for field in message.fields() {
        if matches!(field.case(), FieldCase::Const(_)) {
            continue;
        }
        let field_name = format_ident!("{}", rust_field_name(field));
        let default_value = default_for_field(field);
        defaults.push(quote! { #field_name: #default_value, });
        let read = read_field_tokens(field, &field_name);
        assignments.push(quote! {
            #field_name: #read,
        });
    }
    DecodeFieldTokens {
        defaults: quote! { #(#defaults)* },
        assignments: quote! { #(#assignments)* },
    }
}

fn read_field_tokens(field: &FieldInfo, _field_name: &proc_macro2::Ident) -> TokenStream {
    match field.case() {
        FieldCase::Vector => {
            let element = read_scalar_or_message_inner(field);
            quote! {
                {
                    if reader.is_exhausted() {
                        Vec::new()
                    } else {
                        let length = reader.read_u32()?;
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

fn read_scalar_or_message(field: &FieldInfo, _nested: bool) -> TokenStream {
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

fn read_scalar_or_message_inner(field: &FieldInfo) -> TokenStream {
    match field.datatype() {
        DataType::String => quote! { reader.read_string()? },
        DataType::Bool => quote! { reader.read_bool()? },
        DataType::U8(_) => quote! { reader.read_u8()? },
        DataType::U16 => quote! { reader.read_u16()? },
        DataType::U32 => quote! { reader.read_u32()? },
        DataType::U64 => quote! { reader.read_u64()? },
        DataType::I8(_) => quote! { reader.read_i8()? },
        DataType::I16 => quote! { reader.read_i16()? },
        DataType::I32 => quote! { reader.read_i32()? },
        DataType::I64 => quote! { reader.read_i64()? },
        DataType::F32 => quote! { reader.read_f32()? },
        DataType::F64 => quote! { reader.read_f64()? },
        DataType::GlobalMessage(path) => {
            let package = format_ident!("{}", path.package());
            let name = format_ident!("{}", path.name());
            quote! { <crate::msg::#package::#name>::cdr_decode_fields(reader)? }
        }
        other => {
            let type_name = other.to_string();
            panic!("unsupported field type {type_name}");
        }
    }
}

fn encode_field_tokens(message: &ros2_message::Msg) -> Vec<TokenStream> {
    let mut tokens = Vec::new();
    for field in message.fields() {
        if matches!(field.case(), FieldCase::Const(_)) {
            continue;
        }
        let field_name = format_ident!("{}", rust_field_name(field));
        tokens.push(write_field_tokens(field, quote! { self.#field_name }));
    }
    tokens
}

fn write_field_tokens(field: &FieldInfo, value: TokenStream) -> TokenStream {
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
        _ => write_scalar_or_message(field, value),
    }
}

fn write_vector_element(field: &FieldInfo) -> TokenStream {
    match field.datatype() {
        DataType::GlobalMessage(path) => {
            let package = format_ident!("{}", path.package());
            let name = format_ident!("{}", path.name());
            quote! {
                <crate::msg::#package::#name>::cdr_encode_fields(element, writer)?;
            }
        }
        DataType::String => quote! { writer.write_string(element.as_str())?; },
        _ => write_scalar_or_message(field, quote! { *element }),
    }
}

fn write_scalar_or_message(field: &FieldInfo, value: TokenStream) -> TokenStream {
    match field.datatype() {
        DataType::String => quote! { writer.write_string(#value.as_str())?; },
        DataType::Bool => quote! { writer.write_bool(#value)?; },
        DataType::U8(_) => quote! { writer.write_u8(#value)?; },
        DataType::U16 => quote! { writer.write_u16(#value)?; },
        DataType::U32 => quote! { writer.write_u32(#value)?; },
        DataType::U64 => quote! { writer.write_u64(#value)?; },
        DataType::I8(_) => quote! { writer.write_i8(#value)?; },
        DataType::I16 => quote! { writer.write_i16(#value)?; },
        DataType::I32 => quote! { writer.write_i32(#value)?; },
        DataType::I64 => quote! { writer.write_i64(#value)?; },
        DataType::F32 => quote! { writer.write_f32(#value)?; },
        DataType::F64 => quote! { writer.write_f64(#value)?; },
        DataType::GlobalMessage(path) => {
            let package = format_ident!("{}", path.package());
            let name = format_ident!("{}", path.name());
            quote! { <crate::msg::#package::#name>::cdr_encode_fields(&#value, writer)?; }
        }
        other => {
            let type_name = other.to_string();
            panic!("unsupported field type {type_name}");
        }
    }
}

fn default_for_field(field: &FieldInfo) -> TokenStream {
    match field.case() {
        FieldCase::Vector => quote! { Vec::new() },
        FieldCase::Array(size) => {
            let element_default = match field.datatype() {
                DataType::GlobalMessage(path) => {
                    let package = format_ident!("{}", path.package());
                    let name = format_ident!("{}", path.name());
                    quote! { <crate::msg::#package::#name>::default() }
                }
                _ => quote! { Default::default() },
            };
            quote! { [#element_default; #size] }
        }
        _ => match field.datatype() {
            DataType::String => quote! { String::new() },
            DataType::Bool => quote! { false },
            DataType::GlobalMessage(path) => {
                let package = format_ident!("{}", path.package());
                let name = format_ident!("{}", path.name());
                quote! { <crate::msg::#package::#name>::default() }
            }
            _ => quote! { Default::default() },
        },
    }
}

fn rust_type_tokens(field: &FieldInfo) -> TokenStream {
    let base = match field.datatype() {
        DataType::String => quote! { String },
        DataType::Bool => quote! { bool },
        DataType::U8(_) => quote! { u8 },
        DataType::U16 => quote! { u16 },
        DataType::U32 => quote! { u32 },
        DataType::U64 => quote! { u64 },
        DataType::I8(_) => quote! { i8 },
        DataType::I16 => quote! { i16 },
        DataType::I32 => quote! { i32 },
        DataType::I64 => quote! { i64 },
        DataType::F32 => quote! { f32 },
        DataType::F64 => quote! { f64 },
        DataType::GlobalMessage(path) => {
            let package = format_ident!("{}", path.package());
            let name = format_ident!("{}", path.name());
            quote! { crate::msg::#package::#name }
        }
        other => {
            let type_name = other.to_string();
            panic!("unsupported field type {type_name}");
        }
    };
    match field.case() {
        FieldCase::Vector => quote! { Vec<#base> },
        FieldCase::Array(size) => quote! { [#base; #size] },
        _ => base,
    }
}

fn const_value_tokens(value: ros2_message::Value) -> TokenStream {
    match value {
        ros2_message::Value::U8(value) => quote! { u8 = #value },
        ros2_message::Value::U16(value) => quote! { u16 = #value },
        ros2_message::Value::U32(value) => quote! { u32 = #value },
        ros2_message::Value::U64(value) => quote! { u64 = #value },
        ros2_message::Value::I8(value) => quote! { i8 = #value },
        ros2_message::Value::I16(value) => quote! { i16 = #value },
        ros2_message::Value::I32(value) => quote! { i32 = #value },
        ros2_message::Value::I64(value) => quote! { i64 = #value },
        ros2_message::Value::F32(value) => quote! { f32 = #value },
        ros2_message::Value::F64(value) => quote! { f64 = #value },
        ros2_message::Value::String(value) => quote! { &'static str = #value },
        _ => quote! { () },
    }
}

fn write_typescript(records: &BTreeMap<String, MessageRecord>, typescript_dir: &Path) {
    let mut interface_blocks = Vec::new();
    let mut schema_entries = Vec::new();
    for record in records.values() {
        let interface_name = record.name.clone();
        let dynamic_message =
            DynamicMsg::new(&record.dynamic_key, &record.source).expect("parse for typescript");
        let message = dynamic_message.msg();
        let mut fields = Vec::new();
        for field in message.fields() {
            if matches!(field.case(), FieldCase::Const(_)) {
                continue;
            }
            let field_name = rust_field_name(field);
            let ts_name = if field_name == "type_" {
                "type".to_string()
            } else {
                field_name
            };
            let ts_type = typescript_type(field);
            fields.push(format!("  {ts_name}: {ts_type};"));
        }
        interface_blocks.push(format!(
            "export interface {interface_name} {{\n{}\n}}\n",
            fields.join("\n")
        ));
        let schema = schema_text(record, records);
        schema_entries.push(format!(
            "  \"{}\": `{}`,",
            record.schema_name,
            schema.replace('`', "\\`")
        ));
    }

    fs::write(
        typescript_dir.join("messages.d.ts"),
        format!("// @generated\n\n{}\n", interface_blocks.join("\n")),
    )
    .expect("write messages.d.ts");
    fs::write(
        typescript_dir.join("schemas.ts"),
        format!(
            "// @generated\nexport const SCHEMAS: Record<string, string> = {{\n{}\n}};\n",
            schema_entries.join("\n")
        ),
    )
    .expect("write schemas.ts");
    fs::write(
        typescript_dir.join("index.ts"),
        "// @generated\nexport * from \"./schemas\";\n",
    )
    .expect("write index.ts");
}

fn typescript_type(field: &FieldInfo) -> String {
    let base = match field.datatype() {
        DataType::String => "string".to_string(),
        DataType::Bool => "boolean".to_string(),
        DataType::U8(_)
        | DataType::U16
        | DataType::U32
        | DataType::U64
        | DataType::I8(_)
        | DataType::I16
        | DataType::I32
        | DataType::I64 => "number".to_string(),
        DataType::F32 | DataType::F64 => "number".to_string(),
        DataType::GlobalMessage(path) => path.name().to_string(),
        other => other.to_string(),
    };
    match field.case() {
        FieldCase::Vector | FieldCase::Array(_) => format!("{base}[]"),
        _ => base,
    }
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
