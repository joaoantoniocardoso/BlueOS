#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StartRecordingCommand {
    pub rotate_if_active: bool,
}
impl CdrStruct for StartRecordingCommand {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            rotate_if_active: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_bool(self.rotate_if_active)?;
        Ok(())
    }
}
impl Message for StartRecordingCommand {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/StartRecordingCommand\n# Opens a new MCAP session (rotate if one is already active).\n\nbool rotate_if_active";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/StartRecordingCommand";
    const TYPE_HASH: &'static str =
        "3f9387f316dff318effbc26590570ba1ea6ef148f3b1c29bccc5f72a657c0135";
}
impl StartRecordingCommand {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
