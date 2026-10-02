#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::string::String;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteRecordingCommand {
    pub path: String,
}
impl CdrStruct for DeleteRecordingCommand {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        Ok(())
    }
}
impl Message for DeleteRecordingCommand {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/DeleteRecordingCommand\n# Rejected while the recording is being written or repaired.\n\nstring path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/DeleteRecordingCommand";
    const TYPE_HASH: &'static str =
        "b80baa05a1ef8df71bca2185c9a3f57ef02116e4b7a45dafdb510999df220d0a";
}
impl DeleteRecordingCommand {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
