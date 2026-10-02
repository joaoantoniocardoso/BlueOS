#![allow(
    clippy::arbitrary_source_item_ordering,
    missing_docs,
    reason = "generated from ROS .msg sources"
)]
pub mod msg;
use crate::message::Message;
/// ROS 2 `.msg` text of any IDL message, keyed by its `SCHEMA_NAME`.
pub fn schema(schema_name: &str) -> Option<&'static str> {
    match schema_name {
        "blueos_msgs/msg/CommandAck" => Some(msg::blueos_msgs::CommandAck::SCHEMA),
        _ => None,
    }
}
