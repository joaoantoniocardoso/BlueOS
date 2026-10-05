#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StartRecordingResult {}
impl CdrStruct for StartRecordingResult {
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
impl Message for StartRecordingResult {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/StartRecording_Result";
    const TYPE_HASH: &'static str =
        "b9c73a4e6d4bf162d1347bce6fb4a2f722cf1a44bed1fbdabf6b69852f678157";
}
impl StartRecordingResult {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
