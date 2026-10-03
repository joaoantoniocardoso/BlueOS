#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepairRecordingFeedback {}
impl CdrStruct for RepairRecordingFeedback {
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
impl Message for RepairRecordingFeedback {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/RepairRecording_Feedback";
    const TYPE_HASH: &'static str =
        "8b15cdfa92672bc36d28658718aa3f4459b8d93751b5402167be97bc12eae727";
}
impl RepairRecordingFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
