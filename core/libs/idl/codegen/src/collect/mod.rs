//! Parsed ROS interfaces and message records for codegen.

use alloc::collections::BTreeSet;
use std::path::{Path, PathBuf};

use roslibrust_codegen::find_and_parse_ros_messages;

use crate::{error::CodegenError, msg_ast::Message, schema::hash_hex};

/// One `.msg` file, or one part of a `.srv` or `.action` file, under an interfaces folder, parsed, with what
/// `api.lock` and the generated code need of it.
#[derive(Clone, Debug)]
pub struct MessageRecord {
    /// Its schema name: `<package>/msg/<Name>`, or for a part `<package>/srv/<Name>_Request` or
    /// `<package>/action/<Name>_Goal`, as in ROS 2.
    pub schema_name: String,
    /// Its `<package>/<Name>` name, without `msg`.
    pub dynamic_key: String,
    /// The ROS 2 package it is in.
    pub package: String,
    /// The Message name, without the package.
    pub name: String,
    /// The `.msg` file it was parsed from.
    pub source_path: PathBuf,
    /// The text of the `.msg` file.
    pub source: String,
    /// Its fields in order, as `name:type` joined by `;`: what `api.lock` records.
    pub field_signature: String,
    /// The SHA-256 of its schema name and field signature, in hex.
    pub type_hash: String,
    /// The schema names of the Messages its fields use.
    pub dependencies: BTreeSet<String>,
    /// Its fields and constants, that the Rust and TypeScript output is generated from.
    pub message: Message,
}

/// One `.srv` or `.action` file, with what the schema text that lists every part needs of it.
#[derive(Clone, Debug)]
pub(crate) struct InterfaceRecord {
    /// Its interface type, `<package>/srv/<Name>` or `<package>/action/<Name>`.
    pub(crate) schema_name: String,
    /// The text of the file, every part in order.
    pub(crate) source: String,
    /// The schema names of the Messages its parts use.
    pub(crate) dependencies: BTreeSet<String>,
}

struct MessageRecordInputs {
    schema_name: String,
    package: String,
    name: String,
    source_path: PathBuf,
    source: String,
    message: Message,
}

/// The schema name, `<package>/msg/<Name>`, of every message under `interfaces_root`.
pub fn message_schema_names(interfaces_root: &Path) -> Result<BTreeSet<String>, CodegenError> {
    Ok(collect_messages_for_test(interfaces_root)?
        .into_iter()
        .map(|record| record.schema_name)
        .collect())
}

/// Every message under `interfaces_root`, sorted by schema name, for the `api.lock` tool and the `blueos-idl` tests.
pub fn collect_messages_for_test(
    interfaces_root: &Path,
) -> Result<Vec<MessageRecord>, CodegenError> {
    collect_interfaces(interfaces_root).map(|(messages, _interfaces)| messages)
}

pub(crate) fn collect_messages(interfaces_root: &Path) -> Result<Vec<MessageRecord>, CodegenError> {
    collect_interfaces(interfaces_root).map(|(messages, _interfaces)| messages)
}

pub(crate) fn collect_interfaces(
    interfaces_root: &Path,
) -> Result<(Vec<MessageRecord>, Vec<InterfaceRecord>), CodegenError> {
    let (parsed_messages, parsed_queries, parsed_actions) =
        find_and_parse_ros_messages(&[interfaces_root.to_path_buf()])
            .map_err(|error| CodegenError::ParseInterfaces(error.to_string()))?;
    let mut parsed_messages: Vec<_> = parsed_messages
        .into_iter()
        .filter(|parsed| {
            parsed
                .path
                .extension()
                .is_some_and(|extension| extension == "msg")
        })
        .map(|parsed| (format!("{}/msg/{}", parsed.package, parsed.name), parsed))
        .collect();
    let mut interfaces = Vec::new();
    let queries = parsed_queries.into_iter().map(|query| {
        let parts = vec![
            ("Request", query.request_type),
            ("Response", query.response_type),
        ];
        ("srv", query.package, query.name, query.source, parts)
    });
    let actions = parsed_actions.into_iter().map(|action| {
        let parts = vec![
            ("Goal", action.goal_type),
            ("Result", action.result_type),
            ("Feedback", action.feedback_type),
        ];
        ("action", action.package, action.name, action.source, parts)
    });
    for (kind, package, name, source, parts) in queries.chain(actions) {
        let mut dependencies = BTreeSet::new();
        for (part, parsed) in parts {
            let message = Message::from_ros_fields(&parsed.fields, &parsed.constants)?;
            dependencies.extend(message.dependencies());
            parsed_messages.push((format!("{package}/{kind}/{name}_{part}"), parsed));
        }
        interfaces.push(InterfaceRecord {
            schema_name: format!("{package}/{kind}/{name}"),
            source,
            dependencies,
        });
    }
    interfaces.sort_by(|left, right| left.schema_name.cmp(&right.schema_name));
    let mut messages = Vec::new();
    for (schema_name, parsed) in parsed_messages {
        let message = Message::from_ros_fields(&parsed.fields, &parsed.constants)?;
        messages.push(message_record(MessageRecordInputs {
            schema_name,
            package: parsed.package,
            name: parsed.name,
            source_path: parsed.path,
            source: parsed.source,
            message,
        }));
    }
    messages.sort_by(|left, right| left.schema_name.cmp(&right.schema_name));
    Ok((messages, interfaces))
}

fn message_record(inputs: MessageRecordInputs) -> MessageRecord {
    let MessageRecordInputs {
        schema_name,
        package,
        name,
        source_path,
        source,
        message,
    } = inputs;
    let field_signature = crate::msg_ast::field_signature(&message);
    let type_hash = hash_hex(&format!("{schema_name}\n{field_signature}"));
    MessageRecord {
        schema_name,
        dynamic_key: format!("{package}/{name}"),
        package,
        name,
        source_path,
        source,
        field_signature,
        type_hash,
        dependencies: message.dependencies(),
        message,
    }
}
