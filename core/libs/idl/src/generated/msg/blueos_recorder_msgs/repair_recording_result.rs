#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepairRecordingResult {
    pub path: String,
}
impl CdrStruct for RepairRecordingResult {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: reader.read_or_default(|reader| reader.read_string())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        Ok(())
    }
}
impl Message for RepairRecordingResult {
    const SCHEMA: &'static str = "# Job result: the recording the Job repaired. Why it failed is the reason of the Job.\nstring path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/RepairRecording_Result";
    const TYPE_HASH: &'static str =
        "b5b71558c8c59e17ffe631de41f5aa2bfb63885109007bcaab06fe679a6887a9";
}
impl RepairRecordingResult {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
