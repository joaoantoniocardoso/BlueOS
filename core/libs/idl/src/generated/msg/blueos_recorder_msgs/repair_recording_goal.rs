#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepairRecordingGoal {
    pub path: String,
}
impl CdrStruct for RepairRecordingGoal {
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
impl Message for RepairRecordingGoal {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/action/RepairRecording\n# The Job type on blueos/v1/recorder/command/RepairRecording: rewrites a STATE_NEEDS_REPAIR recording so it has a\n# summary again. CancelJob stops it and leaves the recording untouched.\n\nstring path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/RepairRecording_Goal";
    const TYPE_HASH: &'static str =
        "db70da2e2d919b4d7e6b60c02c8d4f5d91ffb7991d5c5d8ec88cd8689aa1d65e";
}
impl RepairRecordingGoal {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
