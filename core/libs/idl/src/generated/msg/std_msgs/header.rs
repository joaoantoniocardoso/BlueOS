#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub stamp: crate::msg::builtin_interfaces::Time,
    pub frame_id: String,
}
impl CdrStruct for Header {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            stamp: reader.read_or_default(|reader| {
                <crate::msg::builtin_interfaces::Time>::cdr_decode_fields(reader)
            })?,
            frame_id: reader.read_or_default(|reader| reader.read_string())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        <crate::msg::builtin_interfaces::Time>::cdr_encode_fields(&self.stamp, writer)?;
        writer.write_string(self.frame_id.as_str())?;
        Ok(())
    }
}
impl Message for Header {
    const SCHEMA: &'static str = "# Standard metadata for higher-level stamped data types.\n\nbuiltin_interfaces/Time stamp\nstring frame_id\n================================================================================\nMSG: builtin_interfaces/Time\n# This message communicates ROS Time defined here:\n# https://design.ros2.org/articles/clock_and_time.html\n\n# The seconds component, valid over all int32 values.\nint32 sec\n\n# The nanoseconds component, valid in the range [0, 1e9).\nuint32 nanosec";
    const SCHEMA_NAME: &'static str = "std_msgs/msg/Header";
    const TYPE_HASH: &'static str =
        "c16e4c4c4091e7613313d7dd2d85cf6f6152452d7b468240a3a41cd3a25db358";
}
impl Header {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
