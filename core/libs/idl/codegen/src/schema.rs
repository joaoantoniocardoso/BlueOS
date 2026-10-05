//! ROS 2 message definition text and dependency ordering for schema lookup.

use alloc::collections::{BTreeMap, BTreeSet};

use crate::{collect::MessageRecord, error::CodegenError, msg_ast::Field};

const SCHEMA_SEPARATOR: &str =
    "================================================================================\n";

enum VisitGate {
    Skip,
    Cycle,
    Proceed,
}

pub(crate) fn rust_field_name(field: &Field) -> String {
    field.name().to_string()
}

pub(crate) fn schema_text(
    schema_name: &str,
    source: &str,
    dependencies: &BTreeSet<String>,
    records: &BTreeMap<String, MessageRecord>,
) -> Result<String, CodegenError> {
    let ordered = ordered_schema_dependencies(schema_name, dependencies, records)?;
    Ok(join_schema_blocks(schema_name, source, &ordered, records))
}

fn ordered_schema_dependencies(
    schema_name: &str,
    dependencies: &BTreeSet<String>,
    records: &BTreeMap<String, MessageRecord>,
) -> Result<Vec<String>, CodegenError> {
    let mut ordered = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for dependency in dependencies {
        visit_schema_dependency(
            dependency,
            records,
            &mut ordered,
            &mut visiting,
            &mut visited,
        )?;
    }
    visit_schema_dependency(
        schema_name,
        records,
        &mut ordered,
        &mut visiting,
        &mut visited,
    )?;
    Ok(ordered)
}

fn visit_gate(
    schema_name: &str,
    visiting: &BTreeSet<String>,
    visited: &BTreeSet<String>,
) -> VisitGate {
    if visited.contains(schema_name) {
        VisitGate::Skip
    } else if visiting.contains(schema_name) {
        VisitGate::Cycle
    } else {
        VisitGate::Proceed
    }
}

fn visit_schema_dependency(
    schema_name: &str,
    records: &BTreeMap<String, MessageRecord>,
    ordered: &mut Vec<String>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> Result<(), CodegenError> {
    match visit_gate(schema_name, visiting, visited) {
        VisitGate::Skip => return Ok(()),
        VisitGate::Cycle => {
            return Err(CodegenError::DependencyCycle {
                schema_name: schema_name.to_string(),
            });
        }
        VisitGate::Proceed => {}
    }
    visiting.insert(schema_name.to_string());
    visit_record_dependencies(schema_name, records, ordered, visiting, visited)?;
    complete_schema_visit(schema_name, ordered, visiting, visited);
    Ok(())
}

fn visit_record_dependencies(
    schema_name: &str,
    records: &BTreeMap<String, MessageRecord>,
    ordered: &mut Vec<String>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> Result<(), CodegenError> {
    let Some(record) = records.get(schema_name) else {
        return Ok(());
    };
    for dependency in &record.dependencies {
        visit_schema_dependency(dependency, records, ordered, visiting, visited)?;
    }
    Ok(())
}

fn complete_schema_visit(
    schema_name: &str,
    ordered: &mut Vec<String>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) {
    visiting.remove(schema_name);
    if visited.contains(schema_name) {
        return;
    }
    ordered.push(schema_name.to_string());
    visited.insert(schema_name.to_string());
}

fn join_schema_blocks(
    schema_name: &str,
    source: &str,
    ordered: &[String],
    records: &BTreeMap<String, MessageRecord>,
) -> String {
    let mut blocks = vec![source.trim_end().to_string()];
    for ordered_name in ordered {
        let Some(message_record) = records.get(ordered_name) else {
            continue;
        };
        if ordered_name == schema_name {
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

pub(crate) fn hash_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(input.as_bytes());
    format!("{:x}", digest)
}
