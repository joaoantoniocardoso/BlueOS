#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StartRecordingFeedback {}
impl CdrStruct for StartRecordingFeedback {
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
impl Message for StartRecordingFeedback {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/StartRecording_Feedback";
    const TYPE_HASH: &'static str =
        "aed0eda94f1eb887df3798c618fdaedab967b00d4e83a1ae286106b08ad724aa";
}
impl StartRecordingFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
