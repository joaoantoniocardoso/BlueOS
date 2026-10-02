#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SetPolicyCommand {
    pub policy: crate::msg::blueos_recorder_msgs::RecordingPolicy,
}
impl CdrStruct for SetPolicyCommand {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            policy: if reader.is_exhausted() {
                <crate::msg::blueos_recorder_msgs::RecordingPolicy>::default()
            } else {
                <crate::msg::blueos_recorder_msgs::RecordingPolicy>::cdr_decode_fields(reader)?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        <crate::msg::blueos_recorder_msgs::RecordingPolicy>::cdr_encode_fields(
            &self.policy,
            writer,
        )?;
        Ok(())
    }
}
impl Message for SetPolicyCommand {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/SetPolicyCommand\n\nblueos_recorder_msgs/RecordingPolicy policy\n================================================================================\nMSG: blueos_recorder_msgs/RecordingPolicy\n# blueos_recorder_msgs/msg/RecordingPolicy\n# Persisted recorder settings (D-11).\n\nbool record_mavlink_only_when_armed\nbool auto_start_recording";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/SetPolicyCommand";
    const TYPE_HASH: &'static str =
        "7fcc2efa83665816d3e550f9b77ba8bc58124dd7039a604ce36237e9ea510737";
}
impl SetPolicyCommand {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
