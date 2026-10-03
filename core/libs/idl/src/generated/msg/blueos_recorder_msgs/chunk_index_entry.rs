#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChunkIndexEntry {
    pub start_time: u64,
    pub end_time: u64,
    pub offset: u64,
    pub length: u64,
    pub compression: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub channel_ids: Vec<u16>,
    pub message_index_length: u64,
}
impl CdrStruct for ChunkIndexEntry {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            start_time: reader.read_or_default(|reader| reader.read_u64())?,
            end_time: reader.read_or_default(|reader| reader.read_u64())?,
            offset: reader.read_or_default(|reader| reader.read_u64())?,
            length: reader.read_or_default(|reader| reader.read_u64())?,
            compression: reader.read_or_default(|reader| reader.read_string())?,
            compressed_size: reader.read_or_default(|reader| reader.read_u64())?,
            uncompressed_size: reader.read_or_default(|reader| reader.read_u64())?,
            channel_ids: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(reader.read_u16()?);
                }
                Ok(values)
            })?,
            message_index_length: reader.read_or_default(|reader| reader.read_u64())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u64(self.start_time)?;
        writer.write_u64(self.end_time)?;
        writer.write_u64(self.offset)?;
        writer.write_u64(self.length)?;
        writer.write_string(self.compression.as_str())?;
        writer.write_u64(self.compressed_size)?;
        writer.write_u64(self.uncompressed_size)?;
        writer.write_u32(self.channel_ids.len() as u32)?;
        for element in self.channel_ids.iter() {
            writer.write_u16(*element)?;
        }
        writer.write_u64(self.message_index_length)?;
        Ok(())
    }
}
impl Message for ChunkIndexEntry {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/ChunkIndexEntry\n\nuint64 start_time\nuint64 end_time\n# Offset and length of the whole Chunk record, header included.\nuint64 offset\nuint64 length\nstring compression\nuint64 compressed_size\nuint64 uncompressed_size\nuint16[] channel_ids\n# Bytes of the MessageIndex records that follow the chunk.\nuint64 message_index_length";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/ChunkIndexEntry";
    const TYPE_HASH: &'static str =
        "7656f7baad23746ec1baa9f61a774ecb3f7a8c8bf4f9b4ce188952be186fa74a";
}
impl ChunkIndexEntry {
    pub const KNOWN_FIELD_COUNT: usize = 9usize;
}
