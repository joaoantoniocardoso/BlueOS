use std::path::PathBuf;

use blueos_idl::Message;
use blueos_idl::msg::blueos_msgs::CommandAck;
use blueos_idl_codegen::collect_messages_for_test;

#[test]
fn schema_lookup_covers_every_interface() {
    let interfaces_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("interfaces");
    for record in collect_messages_for_test(&interfaces_root) {
        assert!(
            blueos_idl::schema(&record.schema_name).is_some(),
            "{} has no schema",
            record.schema_name
        );
    }
    assert_eq!(
        blueos_idl::schema(CommandAck::SCHEMA_NAME),
        Some(CommandAck::SCHEMA)
    );
    assert_eq!(blueos_idl::schema("unknown/msg/Missing"), None);
}

#[test]
fn command_ack_matches_the_frontend_codec_bytes() {
    // Same bytes as RUST_COMMAND_ACK_ROUND_TRIP in core/frontend/tests/blueos-api/cdr.test.ts.
    let expected = [
        0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x71, 0x75, 0x65, 0x75, 0x65, 0x64,
        0x00,
    ];
    let message = CommandAck {
        accepted: true,
        job_id: 42,
        reason: "queued".into(),
    };
    assert_eq!(message.encode().expect("encode"), expected);
    assert_eq!(CommandAck::decode(&expected).expect("decode"), message);
}

#[test]
fn command_ack_round_trip() {
    let message = CommandAck {
        accepted: true,
        job_id: 42,
        reason: "queued".into(),
    };
    let payload = message.encode().expect("encode");
    let decoded = CommandAck::decode(&payload).expect("decode");
    assert_eq!(message, decoded);
}

#[test]
fn command_ack_decode_old_writer_new_reader() {
    let mut writer = blueos_idl::cdr::Writer::new();
    writer.write_bool(true).expect("bool");
    writer.write_u64(7).expect("job id");
    let payload = writer.finish_with_encapsulation();
    let decoded = CommandAck::decode(&payload).expect("decode");
    assert!(decoded.accepted);
    assert_eq!(decoded.job_id, 7);
    assert!(decoded.reason.is_empty());
}
