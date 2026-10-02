#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChannelMessageCount {
    pub channel_id: u16,
    pub count: u64,
}
impl CdrStruct for ChannelMessageCount {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            channel_id: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u16()?
            },
            count: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u16(self.channel_id)?;
        writer.write_u64(self.count)?;
        Ok(())
    }
}
impl Message for ChannelMessageCount {
    const SCHEMA: &'static str =
        "# blueos_recorder_msgs/msg/ChannelMessageCount\n\nuint16 channel_id\nuint64 count";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/ChannelMessageCount";
    const TYPE_HASH: &'static str =
        "179ac4beb6396ec28ad16f0d97386cdc19a35b4cd7afc79b38d38f2ef4761fdd";
}
impl ChannelMessageCount {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
