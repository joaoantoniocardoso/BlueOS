#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SetLevelFeedback {
    pub level: u8,
}
impl CdrStruct for SetLevelFeedback {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            level: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u8()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(self.level)?;
        Ok(())
    }
}
impl Message for SetLevelFeedback {
    const SCHEMA: &'static str = "# Feedback: the level the pump is at.\nuint8 level";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/action/SetLevel_Feedback";
    const TYPE_HASH: &'static str =
        "8ef7679800c0a3fe76127625252faf723051ca530ec9badeaf5790312692c354";
}
impl SetLevelFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
