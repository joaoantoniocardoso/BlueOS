#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRecordingFeedback {}
impl CdrStruct for SnapshotRecordingFeedback {
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
impl Message for SnapshotRecordingFeedback {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/SnapshotRecording_Feedback";
    const TYPE_HASH: &'static str =
        "d12cee16eef15e5458982c39cfe7c40b217969992ababf4a0838e1e09da804bd";
}
impl SnapshotRecordingFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
