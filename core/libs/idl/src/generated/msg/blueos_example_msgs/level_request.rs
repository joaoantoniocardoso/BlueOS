#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LevelRequest {}
impl CdrStruct for LevelRequest {
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
impl Message for LevelRequest {
    const SCHEMA: &'static str = "# blueos_example_msgs/srv/Level\n# The Query on blueos/v1/example/query/Level. Its request is empty.";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/srv/Level_Request";
    const TYPE_HASH: &'static str =
        "c583452d43f567e398f981b243706027c79a7627c35591fc2a2065b213c76e72";
}
impl LevelRequest {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
