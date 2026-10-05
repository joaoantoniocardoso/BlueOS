//! `MessageBySchema` coverage helpers.

use blueos_idl_codegen::{message_by_schema_declares_type, missing_message_by_schema_entries};

#[test]
fn missing_entries_detects_absent_schema_line() {
    let schemas = "  \"pkg/msg/Foo\": `text`,";
    let messages = "export interface MessageBySchema {\n}\n";
    assert_eq!(
        missing_message_by_schema_entries(schemas, messages),
        vec!["pkg/msg/Foo".to_owned()]
    );
    assert!(!message_by_schema_declares_type(messages, "pkg/msg/Foo"));
}
