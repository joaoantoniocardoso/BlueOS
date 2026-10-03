#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepairRecordingResult {}
impl CdrStruct for RepairRecordingResult {
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
impl Message for RepairRecordingResult {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/RepairRecording_Result";
    const TYPE_HASH: &'static str =
        "afdb4ee88ff3cb16c5a7df7451a2a35ca36ac19f7f9bd0a22acf4de0704221ca";
}
impl RepairRecordingResult {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
