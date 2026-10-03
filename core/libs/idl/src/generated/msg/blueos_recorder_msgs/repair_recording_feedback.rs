#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepairRecordingFeedback {
    pub bytes_processed: u64,
    pub total_bytes: u64,
}
impl CdrStruct for RepairRecordingFeedback {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            bytes_processed: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            total_bytes: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u64(self.bytes_processed)?;
        writer.write_u64(self.total_bytes)?;
        Ok(())
    }
}
impl Message for RepairRecordingFeedback {
    const SCHEMA: &'static str = "# Feedback: how far the repair has read into the recording.\nuint64 bytes_processed\nuint64 total_bytes";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/RepairRecording_Feedback";
    const TYPE_HASH: &'static str =
        "a7785a6c215252c35f671a7b190e9b7a5847a5dff49963e8a86b59c698538523";
}
impl RepairRecordingFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
