#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingBytesRequest {
    pub path: String,
    pub offset: u64,
    pub length: u32,
    pub from_end: bool,
}
impl CdrStruct for RecordingBytesRequest {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            offset: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            length: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u32()?
            },
            from_end: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        writer.write_u64(self.offset)?;
        writer.write_u32(self.length)?;
        writer.write_bool(self.from_end)?;
        Ok(())
    }
}
impl Message for RecordingBytesRequest {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/srv/RecordingBytes\n# The Query on blueos/v1/recorder/query/bytes. Its response is a byte range of a recording, so the browser can read\n# a recording over the backbone instead of HTTP ranges.\n\nstring path\nuint64 offset\n# Bytes wanted from `offset`. A response holds at most 1048576 (1 MiB), and fewer at the end of the file.\nuint32 length\n# Read the last `length` bytes instead, ignoring `offset`, so one query gives the size and the MCAP footer.\nbool from_end";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/srv/RecordingBytes_Request";
    const TYPE_HASH: &'static str =
        "a455429883b223ec3c8b4a00f1c2c7fadd74532c1e4bd2f0904cac829eade331";
}
impl RecordingBytesRequest {
    pub const KNOWN_FIELD_COUNT: usize = 4usize;
}
