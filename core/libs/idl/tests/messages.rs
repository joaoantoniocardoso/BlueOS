use blueos_idl::Message;
use blueos_idl::msg::blueos_msgs::CommandAck;

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
