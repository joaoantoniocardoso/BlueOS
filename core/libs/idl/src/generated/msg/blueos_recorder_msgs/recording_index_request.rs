#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingIndexRequest {
    pub path: String,
    pub from_offset: u64,
    pub limit: u32,
}
impl CdrStruct for RecordingIndexRequest {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: reader.read_or_default(|reader| reader.read_string())?,
            from_offset: reader.read_or_default(|reader| reader.read_u64())?,
            limit: reader.read_or_default(|reader| reader.read_u32())?,
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
    const SCHEMA: &'static str = "# blueos_recorder_msgs/srv/RecordingIndex\n# The Query on blueos/v1/recorder/query/index. Its response is one page of a walk over record headers, so the\n# browser can fetch chunk bodies with HTTP ranges even when the file has no summary yet (still recording, needs\n# repair).\n\nstring path\n# 0 starts at the file magic; otherwise the `offset` of a previous response.\nuint64 from_offset\n# Maximum chunks in the response (1..=20000).\nuint32 limit";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/srv/RecordingIndex_Request";
    const TYPE_HASH: &'static str =
        "7916e0018ff26735e04e34cc544ba70e54ca49bed083e21ca88413299d38ecbe";
}
impl RecordingIndexRequest {
    pub const KNOWN_FIELD_COUNT: usize = 3usize;
}
