#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::string::String;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingIndexRequest {
    pub path: String,
    pub from_offset: u64,
    pub limit: u32,
}
impl CdrStruct for RecordingIndexRequest {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            from_offset: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            limit: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u32()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        writer.write_u64(self.from_offset)?;
        writer.write_u32(self.limit)?;
        Ok(())
    }
}
impl Message for RecordingIndexRequest {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/RecordingIndexRequest\n# Payload of the blueos/v1/recorder/query/index query.\n\nstring path\n# 0 starts at the file magic; otherwise the `offset` of a previous RecordingIndex reply.\nuint64 from_offset\n# Maximum chunks in the reply (1..=20000).\nuint32 limit";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/RecordingIndexRequest";
    const TYPE_HASH: &'static str =
        "c4c436628167efca35d621b95cdf659343a3ce2f2b5709ce1fe34efd329bc11e";
}
impl RecordingIndexRequest {
    pub const KNOWN_FIELD_COUNT: usize = 3usize;
}
