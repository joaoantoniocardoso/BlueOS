//! The committed TypeScript names the type of every embedded schema, so the frontend decodes typed Messages.

const MESSAGES: &str = include_str!("../typescript/messages.d.ts");
const SCHEMAS: &str = include_str!("../typescript/schemas.ts");

#[test]
fn every_embedded_schema_names_its_message_type() {
    let schema_names: Vec<&str> = SCHEMAS
        .lines()
        .filter_map(|line| line.strip_prefix("  \"")?.split_once("\": `"))
        .map(|(schema_name, _)| schema_name)
        .collect();

    assert!(schema_names.contains(&"blueos_msgs/msg/CommandAck"));
    assert!(MESSAGES.contains("export interface MessageBySchema {\n"));
    for schema_name in schema_names {
        // A part is named `<Name>_Request` in ROS 2 and `<Name>Request` in TypeScript.
        let type_name = schema_name
            .rsplit('/')
            .next()
            .unwrap_or(schema_name)
            .replace('_', "");
        assert!(
            MESSAGES.contains(&format!("  \"{schema_name}\": {type_name};\n")),
            "MessageBySchema has no {schema_name}; run: cargo run -p blueos-idl-codegen -- --write"
        );
    }
}
