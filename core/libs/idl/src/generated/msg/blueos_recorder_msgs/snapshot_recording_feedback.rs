#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRecordingFeedback {
    pub output_path: String,
}
impl CdrStruct for SnapshotRecordingFeedback {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            output_path: reader.read_or_default(|reader| reader.read_string())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.output_path.as_str())?;
        Ok(())
    }
}
impl Message for SnapshotRecordingFeedback {
    const SCHEMA: &'static str = "# Feedback: the snapshot the Job is writing.\nstring output_path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/SnapshotRecording_Feedback";
    const TYPE_HASH: &'static str =
        "825130f67f40a47d1bf4978f8aa58476aa6bd54cdb42b8871de7e3ce9d6f0bc8";
}
impl SnapshotRecordingFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
