//! The committed TypeScript names the type of every embedded schema, so the frontend decodes typed Messages.

use blueos_idl_codegen::{message_by_schema_declares_type, missing_message_by_schema_entries};

const MESSAGES: &str = include_str!("../typescript/messages.d.ts");
const SCHEMAS: &str = include_str!("../typescript/schemas.ts");

#[test]
fn every_embedded_schema_names_its_message_type() {
    assert!(message_by_schema_declares_type(
        MESSAGES,
        "blueos_msgs/msg/CommandAck"
    ));
    assert!(MESSAGES.contains("export interface MessageBySchema {\n"));
    let missing = missing_message_by_schema_entries(SCHEMAS, MESSAGES);
    assert!(
        missing.is_empty(),
        "MessageBySchema is missing: {missing:?}; run: cargo run -p blueos-idl-codegen -- --write"
    );
}
