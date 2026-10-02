#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepairRecordingCommand {
    pub path: String,
}
impl CdrStruct for RepairRecordingCommand {
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
impl Message for RepairRecordingCommand {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/RepairRecordingCommand\n# Rewrites a STATE_NEEDS_REPAIR recording so it has a summary again. Progress is on the library state.\n\nstring path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/RepairRecordingCommand";
    const TYPE_HASH: &'static str =
        "cbdd4409db04db54173ae0b040879b41dbb0bcf1c533a6aee5f27b69b5cff076";
}
impl RepairRecordingCommand {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
