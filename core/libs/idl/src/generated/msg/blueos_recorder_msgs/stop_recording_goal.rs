#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopRecordingGoal {}
impl CdrStruct for StopRecordingGoal {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        if !reader.is_exhausted() {
            reader.read_u8()?;
        }
        Ok(Self {})
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(0)?;
        Ok(())
    }
}
impl Message for StopRecordingGoal {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/action/StopRecording\n# The Job type on blueos/v1/recorder/command/Stop: finishes the current MCAP session; samples are dropped until\n# Start.";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/action/StopRecording_Goal";
    const TYPE_HASH: &'static str =
        "872e715368d1a4df8c5dfe62bb2a5b972288d1a5c9b0191f1a0578faea99ef38";
}
impl StopRecordingGoal {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
