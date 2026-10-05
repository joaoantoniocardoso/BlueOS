#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteRecordingGoal {
    pub path: String,
}
impl CdrStruct for DeleteRecordingGoal {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: reader.read_or_default(|reader| reader.read_string())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        Ok(())
    }
}
impl Message for DeleteRecordingGoal {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/action/DeleteRecording\n# The Job type on blueos/v1/recorder/command/DeleteRecording. Rejected while the recording is being written or\n# repaired. The Job succeeds once the file is gone.\n\nstring path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/DeleteRecording_Goal";
    const TYPE_HASH: &'static str =
        "28a0a34912e735f751fed630a5ec1eec10049f8286a3f0b20a2b2b6629a0a98d";
}
impl DeleteRecordingGoal {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
