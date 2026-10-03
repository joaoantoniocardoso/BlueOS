#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteRecordingResult {
    pub path: String,
}
impl CdrStruct for DeleteRecordingResult {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        Ok(())
    }
}
impl Message for DeleteRecordingResult {
    const SCHEMA: &'static str = "# Job result: the recording the Job deleted. Why it failed is the reason of the Job.\nstring path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/DeleteRecording_Result";
    const TYPE_HASH: &'static str =
        "4c8dae00ff590cd12daac0426a8660ec78823c2a051e15e419080fd164fd4e06";
}
impl DeleteRecordingResult {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
