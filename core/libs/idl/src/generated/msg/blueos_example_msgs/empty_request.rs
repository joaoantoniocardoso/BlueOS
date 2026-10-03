#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EmptyRequest {}
impl CdrStruct for EmptyRequest {
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
impl Message for EmptyRequest {
    const SCHEMA: &'static str = "# blueos_example_msgs/msg/EmptyRequest\n# Command payload with no semantics (StartSelfTest, CancelSelfTest). Declared empty, with no placeholder field.";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/msg/EmptyRequest";
    const TYPE_HASH: &'static str =
        "f1734b149172a687e3f4961e15fbacbdf1b343df7220696f80befce499ba3334";
}
impl EmptyRequest {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
