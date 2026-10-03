#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
pub mod constants_log {
    pub const UNKNOWN: u8 = 0u8;
    pub const DEBUG: u8 = 1u8;
    pub const INFO: u8 = 2u8;
    pub const WARNING: u8 = 3u8;
    pub const ERROR: u8 = 4u8;
    pub const FATAL: u8 = 5u8;
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Log {
    pub timestamp: crate::msg::builtin_interfaces::Time,
    pub level: u8,
    pub message: String,
    pub name: String,
    pub file: String,
    pub line: u32,
}
impl CdrStruct for Log {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            timestamp: reader.read_or_default(|reader| {
                <crate::msg::builtin_interfaces::Time>::cdr_decode_fields(reader)
            })?,
            level: reader.read_or_default(|reader| reader.read_u8())?,
            message: reader.read_or_default(|reader| reader.read_string())?,
            name: reader.read_or_default(|reader| reader.read_string())?,
            file: reader.read_or_default(|reader| reader.read_string())?,
            line: reader.read_or_default(|reader| reader.read_u32())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        <crate::msg::builtin_interfaces::Time>::cdr_encode_fields(&self.timestamp, writer)?;
        writer.write_u8(self.level)?;
        writer.write_string(self.message.as_str())?;
        writer.write_string(self.name.as_str())?;
        writer.write_string(self.file.as_str())?;
        writer.write_u32(self.line)?;
        Ok(())
    }
}
impl Message for Log {
    const SCHEMA: &'static str = "# foxglove_msgs/msg/Log\n# A log message for service tracing output (D-13).\n\nbuiltin_interfaces/Time timestamp\n\nuint8 UNKNOWN=0\nuint8 DEBUG=1\nuint8 INFO=2\nuint8 WARNING=3\nuint8 ERROR=4\nuint8 FATAL=5\n\nuint8 level\nstring message\nstring name\nstring file\nuint32 line\n================================================================================\nMSG: builtin_interfaces/Time\n# This message communicates ROS Time defined here:\n# https://design.ros2.org/articles/clock_and_time.html\n\n# The seconds component, valid over all int32 values.\nint32 sec\n\n# The nanoseconds component, valid in the range [0, 1e9).\nuint32 nanosec";
    const SCHEMA_NAME: &'static str = "foxglove_msgs/msg/Log";
    const TYPE_HASH: &'static str =
        "8076caa2c7cebac7d915ff8dc582f70f8eb37bb52f5b7b849a013d8f58798f08";
}
impl Log {
    pub const KNOWN_FIELD_COUNT: usize = 6usize;
}
