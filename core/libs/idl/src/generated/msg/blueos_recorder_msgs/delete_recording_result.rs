#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteRecordingResult {}
impl CdrStruct for DeleteRecordingResult {
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
impl Message for DeleteRecordingResult {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/DeleteRecording_Result";
    const TYPE_HASH: &'static str =
        "c9d8e9bb0f14430494bf4c066101b1451cb4bb099a3c0a2619f6dc4ca2e24de0";
}
impl DeleteRecordingResult {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
