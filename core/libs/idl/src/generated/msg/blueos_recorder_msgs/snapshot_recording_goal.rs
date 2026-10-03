#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRecordingGoal {
    pub path: String,
}
impl CdrStruct for SnapshotRecordingGoal {
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
impl Message for SnapshotRecordingGoal {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/action/SnapshotRecording\n# The Job type on blueos/v1/recorder/command/SnapshotRecording: writes an indexed copy of a recording (typically the\n# one being written) next to it, named <stem>.snapshot-<UTC ISO time>Z.mcap. The copy is announced by a\n# RecordingOperation event.\n\nstring path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/SnapshotRecording_Goal";
    const TYPE_HASH: &'static str =
        "31087aec0a77e1fdc43a4a2af6b1e1c4b29c089c385d7ebf62101ae360233927";
}
impl SnapshotRecordingGoal {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
