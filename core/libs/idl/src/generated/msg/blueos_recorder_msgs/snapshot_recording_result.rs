#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRecordingResult {
    pub path: String,
    pub output_path: String,
}
impl CdrStruct for SnapshotRecordingResult {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            output_path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        writer.write_string(self.output_path.as_str())?;
        Ok(())
    }
}
impl Message for SnapshotRecordingResult {
    const SCHEMA: &'static str = "# Job result: the recording the Job copied, and the snapshot, which exists when the Job succeeded.\nstring path\nstring output_path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/SnapshotRecording_Result";
    const TYPE_HASH: &'static str =
        "7cedc0486d39529896138824b871908ca4950a5fb7313e70fc3897095b7413d7";
}
impl SnapshotRecordingResult {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
