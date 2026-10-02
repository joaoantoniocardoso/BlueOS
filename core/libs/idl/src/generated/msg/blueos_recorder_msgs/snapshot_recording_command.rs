#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRecordingCommand {
    pub path: String,
}
impl CdrStruct for SnapshotRecordingCommand {
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
impl Message for SnapshotRecordingCommand {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/SnapshotRecordingCommand\n# Writes an indexed copy of a recording (typically the one being written) next to it, named\n# <stem>.snapshot-<UTC ISO time>Z.mcap. The copy is announced by a RecordingOperation event.\n\nstring path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/SnapshotRecordingCommand";
    const TYPE_HASH: &'static str =
        "9bd651ccdd85935b3f00b6dcd05127589a9ccdd5f7aec99ad50d003452925507";
}
impl SnapshotRecordingCommand {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
