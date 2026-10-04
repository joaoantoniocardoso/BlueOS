#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingBytesResponse {
    pub size: u64,
    pub data: Vec<u8>,
}
impl CdrStruct for RecordingBytesResponse {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            size: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            data: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    reader.read_bytes(length as usize)?.to_vec()
                }
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u64(self.size)?;
        writer.write_u32(self.data.len() as u32)?;
        writer.write_bytes(&self.data)?;
        Ok(())
    }
}
impl Message for RecordingBytesResponse {
    const SCHEMA: &'static str = "# The size of the file when it was read; it grows while the recording is written.\nuint64 size\nuint8[] data";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/srv/RecordingBytes_Response";
    const TYPE_HASH: &'static str =
        "6f99ce2c0370682652b9ee22b17f8cc002ceb023b23d0bfb367f188e2f0d910b";
}
impl RecordingBytesResponse {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
