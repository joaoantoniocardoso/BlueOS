#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingIndexResponse {
    pub size: u64,
    pub offset: u64,
    pub closed: bool,
    pub chunks: Vec<crate::msg::blueos_recorder_msgs::ChunkIndexEntry>,
    pub message_counts: Vec<crate::msg::blueos_recorder_msgs::ChannelMessageCount>,
    pub records: Vec<u8>,
}
impl CdrStruct for RecordingIndexResponse {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            size: reader.read_or_default(|reader| reader.read_u64())?,
            offset: reader.read_or_default(|reader| reader.read_u64())?,
            closed: reader.read_or_default(|reader| reader.read_bool())?,
            chunks: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(
                        <crate::msg::blueos_recorder_msgs::ChunkIndexEntry>::cdr_decode_fields(
                            reader,
                        )?,
                    );
                }
                Ok(values)
            })?,
            message_counts: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(
                        <crate::msg::blueos_recorder_msgs::ChannelMessageCount>::cdr_decode_fields(
                            reader,
                        )?,
                    );
                }
                Ok(values)
            })?,
            records: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(reader.read_u8()?);
                }
                Ok(values)
            })?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u64(self.size)?;
        writer.write_u64(self.offset)?;
        writer.write_bool(self.closed)?;
        writer.write_u32(self.chunks.len() as u32)?;
        for element in self.chunks.iter() {
            <crate::msg::blueos_recorder_msgs::ChunkIndexEntry>::cdr_encode_fields(
                element, writer,
            )?;
        }
        writer.write_u32(self.message_counts.len() as u32)?;
        for element in self.message_counts.iter() {
            <crate::msg::blueos_recorder_msgs::ChannelMessageCount>::cdr_encode_fields(
                element, writer,
            )?;
        }
        writer.write_u32(self.records.len() as u32)?;
        for element in self.records.iter() {
            writer.write_u8(*element)?;
        }
        Ok(())
    }
}
impl Message for RecordingIndexResponse {
    const SCHEMA: &'static str = "uint64 size\n# Where the next page starts.\nuint64 offset\n# A DataEnd or Footer record was reached: the walk is complete.\nbool closed\nblueos_recorder_msgs/ChunkIndexEntry[] chunks\nblueos_recorder_msgs/ChannelMessageCount[] message_counts\n# Raw Header, Schema, Channel and Metadata records met in this page, in file order.\nuint8[] records\n================================================================================\nMSG: blueos_recorder_msgs/ChannelMessageCount\n# blueos_recorder_msgs/msg/ChannelMessageCount\n\nuint16 channel_id\nuint64 count\n================================================================================\nMSG: blueos_recorder_msgs/ChunkIndexEntry\n# blueos_recorder_msgs/msg/ChunkIndexEntry\n\nuint64 start_time\nuint64 end_time\n# Offset and length of the whole Chunk record, header included.\nuint64 offset\nuint64 length\nstring compression\nuint64 compressed_size\nuint64 uncompressed_size\nuint16[] channel_ids\n# Bytes of the MessageIndex records that follow the chunk.\nuint64 message_index_length";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/srv/RecordingIndex_Response";
    const TYPE_HASH: &'static str =
        "200e518341b652bd029a3aa03d1b8c5a6f0875dcc5937fbd2270d4a92080791a";
}
impl RecordingIndexResponse {
    pub const KNOWN_FIELD_COUNT: usize = 6usize;
}
