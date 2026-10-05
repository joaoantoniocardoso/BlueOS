#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopRecordingFeedback {}
impl CdrStruct for StopRecordingFeedback {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        if !reader.is_exhausted() {
            reader.read_u8()?;
        }
        Ok(Self {})
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(0)?;
        Ok(())
    }
}
impl Message for StopRecordingFeedback {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/StopRecording_Feedback";
    const TYPE_HASH: &'static str =
        "da129cd981015fdf9439fd2460dc5744859a5f40bfa3d227253f4a46ac0f06db";
}
impl StopRecordingFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
