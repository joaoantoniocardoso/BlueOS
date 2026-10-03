#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Time {
    pub sec: i32,
    pub nanosec: u32,
}
impl CdrStruct for Time {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            sec: reader.read_or_default(|reader| reader.read_i32())?,
            nanosec: reader.read_or_default(|reader| reader.read_u32())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_i32(self.sec)?;
        writer.write_u32(self.nanosec)?;
        Ok(())
    }
}
impl Message for Time {
    const SCHEMA: &'static str = "# This message communicates ROS Time defined here:\n# https://design.ros2.org/articles/clock_and_time.html\n\n# The seconds component, valid over all int32 values.\nint32 sec\n\n# The nanoseconds component, valid in the range [0, 1e9).\nuint32 nanosec";
    const SCHEMA_NAME: &'static str = "builtin_interfaces/msg/Time";
    const TYPE_HASH: &'static str =
        "f979b87a898619d092c19df7c74f58724d5c1558e643b733b5f9b201d6320af5";
}
impl Time {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
