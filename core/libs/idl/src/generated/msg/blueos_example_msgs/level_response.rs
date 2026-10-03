#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LevelResponse {
    pub level: u8,
    pub max_level: u8,
}
impl CdrStruct for LevelResponse {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            level: reader.read_or_default(|reader| reader.read_u8())?,
            max_level: reader.read_or_default(|reader| reader.read_u8())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(self.level)?;
        writer.write_u8(self.max_level)?;
        Ok(())
    }
}
impl Message for LevelResponse {
    const SCHEMA: &'static str = "uint8 level\nuint8 max_level";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/srv/Level_Response";
    const TYPE_HASH: &'static str =
        "a6e5f89f3aaae348bd738ca7067e44cf9067fba46e80bd9db1badc121c046560";
}
impl LevelResponse {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
