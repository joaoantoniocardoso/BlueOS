#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SetLevelGoal {
    pub level: u8,
}
impl CdrStruct for SetLevelGoal {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            level: reader.read_or_default(|reader| reader.read_u8())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(self.level)?;
        Ok(())
    }
}
impl Message for SetLevelGoal {
    const SCHEMA: &'static str = "# blueos_example_msgs/action/SetLevel\n# The Job type on blueos/v1/example/command/SetLevel: the pump moves to a level one step at a time.\n\n# Goal: the level to reach, at most PumpState.max_level.\nuint8 level";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/action/SetLevel_Goal";
    const TYPE_HASH: &'static str =
        "54a3a0857fb9a066e85131dfa67a1565ad001b753c2e3d79b5e9cf76bde19350";
}
impl SetLevelGoal {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
