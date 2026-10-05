#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StartRecordingGoal {
    pub rotate_if_active: bool,
}
impl CdrStruct for StartRecordingGoal {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            rotate_if_active: reader.read_or_default(|reader| reader.read_bool())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_bool(self.rotate_if_active)?;
        Ok(())
    }
}
impl Message for StartRecordingGoal {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/action/StartRecording\n# The Job type on blueos/v1/recorder/command/Start: opens a new MCAP session (rotate if one is already active).\n\nbool rotate_if_active";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/StartRecording_Goal";
    const TYPE_HASH: &'static str =
        "03c4c6bebf5e4d21e8cc536f27e8a090b0fc8272e04b778edb8ad1584a930e02";
}
impl StartRecordingGoal {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
