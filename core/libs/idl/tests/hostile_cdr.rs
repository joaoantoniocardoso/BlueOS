//! Hostile CDR inputs must fail without large allocations.

use blueos_idl::{Message, cdr, msg::blueos_msgs::ServiceInfo};

#[test]
fn service_info_rejects_hostile_capabilities_length() {
    let mut writer = cdr::Writer::new();
    writer.write_string("recorder").expect("name");
    writer.write_string("1.0.0").expect("version");
    writer.write_string("dev").expect("build");
    writer.write_u32(0xFFFF_FFFF).expect("capabilities length");
    let payload = writer.finish_with_encapsulation();
    assert!(ServiceInfo::decode(&payload).is_err());
}
