#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LevelQueryResponse {
    pub level: u8,
    pub max_level: u8,
}
impl CdrStruct for LevelQueryResponse {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            level: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u8()?
            },
            max_level: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u8()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(self.level)?;
        writer.write_u8(self.max_level)?;
        Ok(())
    }
}
impl Message for LevelQueryResponse {
    const SCHEMA: &'static str = "# blueos_example_msgs/msg/LevelQueryResponse\n# Reply for blueos/v1/example/query/Level.\n\nuint8 level\nuint8 max_level";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/msg/LevelQueryResponse";
    const TYPE_HASH: &'static str =
        "e8078ae454d2cee17351cc1656144ce07ee0c99b6635fbf434151a7aa2385433";
}
impl LevelQueryResponse {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
