#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SetLevelResult {
    pub level: u8,
}
impl CdrStruct for SetLevelResult {
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
impl Message for SetLevelResult {
    const SCHEMA: &'static str = "# Job result: the level the pump reached.\nuint8 level";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/action/SetLevel_Result";
    const TYPE_HASH: &'static str =
        "93248625faf5b31b9ab77c404269742bd0b3c7c2c9cdd1391e09386e025b8c1f";
}
impl SetLevelResult {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
