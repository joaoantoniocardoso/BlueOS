#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingIndex {
    pub size: u64,
    pub offset: u64,
    pub closed: bool,
    pub chunks: Vec<crate::msg::blueos_recorder_msgs::ChunkIndexEntry>,
    pub message_counts: Vec<crate::msg::blueos_recorder_msgs::ChannelMessageCount>,
    pub records: Vec<u8>,
}
impl CdrStruct for RecordingIndex {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            size: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            offset: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            closed: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
            chunks: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(
                            <crate::msg::blueos_recorder_msgs::ChunkIndexEntry>::cdr_decode_fields(
                                reader,
                            )?,
                        );
                    }
                    values
                }
            },
            message_counts: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values
                            .push(
                                <crate::msg::blueos_recorder_msgs::ChannelMessageCount>::cdr_decode_fields(
                                    reader,
                                )?,
                            );
                    }
                    values
                }
            },
            records: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(reader.read_u8()?);
                    }
                    values
                }
            },
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
impl Message for RecordingIndex {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/RecordingIndex\n# Reply of the index query: one page of a walk over record headers, so the browser can fetch chunk\n# bodies with HTTP ranges even when the file has no summary yet (still recording, needs repair).\n\nuint64 size\n# Where the next page starts.\nuint64 offset\n# A DataEnd or Footer record was reached: the walk is complete.\nbool closed\nblueos_recorder_msgs/ChunkIndexEntry[] chunks\nblueos_recorder_msgs/ChannelMessageCount[] message_counts\n# Raw Header, Schema, Channel and Metadata records met in this page, in file order.\nuint8[] records\n================================================================================\nMSG: blueos_recorder_msgs/ChannelMessageCount\n# blueos_recorder_msgs/msg/ChannelMessageCount\n\nuint16 channel_id\nuint64 count\n================================================================================\nMSG: blueos_recorder_msgs/ChunkIndexEntry\n# blueos_recorder_msgs/msg/ChunkIndexEntry\n\nuint64 start_time\nuint64 end_time\n# Offset and length of the whole Chunk record, header included.\nuint64 offset\nuint64 length\nstring compression\nuint64 compressed_size\nuint64 uncompressed_size\nuint16[] channel_ids\n# Bytes of the MessageIndex records that follow the chunk.\nuint64 message_index_length";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/RecordingIndex";
    const TYPE_HASH: &'static str =
        "a950ba66197d84e10b2480bc2856ccf6dff3ea6e8ceb5f18959623c3df846521";
}
impl RecordingIndex {
    pub const KNOWN_FIELD_COUNT: usize = 6usize;
}
