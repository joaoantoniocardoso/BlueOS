#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopRecordingCommand {
    pub reserved: u8,
}
impl CdrStruct for StopRecordingCommand {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            reserved: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u8()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(self.reserved)?;
        Ok(())
    }
}
impl Message for StopRecordingCommand {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/StopRecordingCommand\n# Finishes the current MCAP session; samples are dropped until StartRecording.\n\nuint8 reserved";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/StopRecordingCommand";
    const TYPE_HASH: &'static str =
        "6b37b53b2d9be781f1786b61d9fd965f831b3a09a7ea003f52e5f73aa6244ba0";
}
impl StopRecordingCommand {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
