#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopRecordingResult {}
impl CdrStruct for StopRecordingResult {
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
impl Message for StopRecordingResult {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/StopRecording_Result";
    const TYPE_HASH: &'static str =
        "aee46e4dbc9b9f4adea35b8ae357ef10cee2c79d78ed67ce58d7c7f8c8c60199";
}
impl StopRecordingResult {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
