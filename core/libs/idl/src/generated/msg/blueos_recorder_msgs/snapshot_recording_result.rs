#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRecordingResult {}
impl CdrStruct for SnapshotRecordingResult {
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
impl Message for SnapshotRecordingResult {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/SnapshotRecording_Result";
    const TYPE_HASH: &'static str =
        "c49c09df76cf7b329940ac041735e9fbfb8f65a26d2931b3fe0f4ff2cb97f6f1";
}
impl SnapshotRecordingResult {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
