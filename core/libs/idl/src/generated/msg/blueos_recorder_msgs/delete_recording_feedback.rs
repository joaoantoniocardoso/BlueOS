#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteRecordingFeedback {}
impl CdrStruct for DeleteRecordingFeedback {
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
impl Message for DeleteRecordingFeedback {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/DeleteRecording_Feedback";
    const TYPE_HASH: &'static str =
        "e786dce371d60e211162f9e42672cfcd586d5f9011785dbae82198710317f6c7";
}
impl DeleteRecordingFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
