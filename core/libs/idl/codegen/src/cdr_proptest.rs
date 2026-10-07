//! Generated proptest strategies and round-trip cases for the CDR codec.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use convert_case::{Case, Casing};

use crate::{
    collect::MessageRecord,
    error::CodegenError,
    msg_ast::{DataType, Field, FieldCase},
    rust_tokens::{constant_families_by_field, enum_ident_for_field},
    schema::rust_field_name,
};

pub(crate) fn write_cdr_proptest(
    records: &BTreeMap<String, MessageRecord>,
    out_dir: &Path,
) -> Result<(), CodegenError> {
    let mut strategies = Vec::new();
    let mut tests = Vec::new();
    for record in topological_order(records) {
        let strategy_fn = strategy_fn_name(record);
        let type_path = format!("blueos_idl::msg::{}::{}", record.package, record.name);
        strategies.push(strategy_body(record, &strategy_fn, &type_path));
        tests.push(round_trip_test(record, &strategy_fn, &type_path));
    }
    let strategies_path = out_dir.join("cdr_proptest_strategies.rs");
    let strategies_contents = format!(
        "// @generated\nuse proptest::{{collection::vec, prelude::*, prop_compose}};\n\nuse blueos_idl::Message;\n\nuse super::cdr_proptest_helpers::{{bounded_bytes, bounded_string, bounded_vec}};\n\n{}\n",
        strategies.join("\n\n")
    );
    fs::write(&strategies_path, strategies_contents)
        .map_err(|source| CodegenError::io(strategies_path, source))?;

    let cases_path = out_dir.join("cdr_proptest_cases.rs");
    let cases_contents = format!("// @generated\nproptest! {{\n{}\n}}\n", tests.join("\n\n"));
    fs::write(&cases_path, cases_contents)
        .map_err(|source| CodegenError::io(cases_path, source))?;
    Ok(())
}

fn topological_order(records: &BTreeMap<String, MessageRecord>) -> Vec<&MessageRecord> {
    let mut ordered = Vec::new();
    let mut visited = BTreeSet::new();
    for schema_name in records.keys() {
        visit_schema(schema_name, records, &mut ordered, &mut visited);
    }
    ordered
}

fn visit_schema<'a>(
    schema_name: &'a str,
    records: &'a BTreeMap<String, MessageRecord>,
    ordered: &mut Vec<&'a MessageRecord>,
    visited: &mut BTreeSet<&'a str>,
) {
    let Some(record) = records
        .get(schema_name)
        .filter(|_| visited.insert(schema_name))
    else {
        return;
    };
    for dependency in &record.dependencies {
        visit_schema(dependency, records, ordered, visited);
    }
    ordered.push(record);
}

fn strategy_fn_name(record: &MessageRecord) -> String {
    format!(
        "{}_{}_strategy",
        record.package,
        record.name.to_case(Case::Snake)
    )
}

fn strategy_body(record: &MessageRecord, strategy_fn: &str, type_path: &str) -> String {
    let families = constant_families_by_field(&record.message);
    let bindings: Vec<String> = record
        .message
        .fields()
        .iter()
        .map(|field| {
            let rust_name = rust_field_name(field);
            format!(
                "{rust_name} in {}",
                field_strategy_tokens(field, record, &families)
            )
        })
        .collect();
    if bindings.is_empty() {
        return format!(
            "pub(crate) fn {strategy_fn}() -> impl Strategy<Value = {type_path}> {{\n    Just({type_path}::default())\n}}"
        );
    }
    let fields: Vec<String> = record
        .message
        .fields()
        .iter()
        .map(rust_field_name)
        .collect();
    format!(
        "prop_compose! {{\n    pub(crate) fn {strategy_fn}()({}) -> {type_path} {{\n        {type_path} {{ {} }}\n    }}\n}}",
        bindings.join(", "),
        fields.join(", ")
    )
}

fn field_strategy_tokens(
    field: &Field,
    record: &MessageRecord,
    families: &BTreeMap<String, crate::constant_family::ConstantFamily>,
) -> String {
    if let Some(family) = families.get(field.name()) {
        let enum_name = enum_ident_for_field(&record.name, &family.field_name);
        let enum_path = format!("blueos_idl::msg::{}::{}", record.package, enum_name);
        return format!("any::<u8>().prop_map({enum_path}::from_raw)");
    }
    match field.case() {
        FieldCase::Vector if is_byte_sequence(field, families) => "bounded_bytes()".to_string(),
        FieldCase::Vector => {
            let element = scalar_or_message_strategy(field);
            format!("bounded_vec({element})")
        }
        FieldCase::Array(size) => {
            let element = scalar_or_message_strategy(field);
            format!(
                "vec({element}, {size}).prop_map(|values| {{\n            let mut array = [Default::default(); {size}];\n            for (index, value) in values.into_iter().enumerate() {{\n                array[index] = value;\n            }}\n            array\n        }})"
            )
        }
        FieldCase::Scalar | FieldCase::Const(_) => scalar_or_message_strategy(field),
    }
}

fn is_byte_sequence(
    field: &Field,
    families: &BTreeMap<String, crate::constant_family::ConstantFamily>,
) -> bool {
    matches!(field.datatype(), DataType::U8) && !families.contains_key(field.name())
}

fn scalar_or_message_strategy(field: &Field) -> String {
    match field.datatype() {
        DataType::String => "bounded_string()".to_string(),
        DataType::Bool => "any::<bool>()".to_string(),
        DataType::U8 => "any::<u8>()".to_string(),
        DataType::U16 => "any::<u16>()".to_string(),
        DataType::U32 => "any::<u32>()".to_string(),
        DataType::U64 => "any::<u64>()".to_string(),
        DataType::I8 => "any::<i8>()".to_string(),
        DataType::I16 => "any::<i16>()".to_string(),
        DataType::I32 => "any::<i32>()".to_string(),
        DataType::I64 => "any::<i64>()".to_string(),
        DataType::F32 => "any::<f32>()".to_string(),
        DataType::F64 => "any::<f64>()".to_string(),
        DataType::GlobalMessage { package, name } => {
            format!("{package}_{}_strategy()", name.to_case(Case::Snake))
        }
    }
}

fn round_trip_test(record: &MessageRecord, strategy_fn: &str, type_path: &str) -> String {
    let test_name = format!(
        "{}_{}_round_trips",
        record.package,
        record.name.to_case(Case::Snake)
    );
    format!(
        "    #[test]\n    fn {test_name}(message in {strategy_fn}()) {{\n        let encoded = message.encode().expect(\"encode\");\n        let decoded = {type_path}::decode(&encoded).expect(\"decode\");\n        prop_assert_eq!(message, decoded);\n    }}"
    )
}
