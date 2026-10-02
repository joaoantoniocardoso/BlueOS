#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SetLevelRequest {
    pub level: u8,
}
impl CdrStruct for SetLevelRequest {
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
impl Message for SetLevelRequest {
    const SCHEMA: &'static str = "# blueos_example_msgs/msg/SetLevelRequest\n# Payload for blueos/v1/example/command/SetLevel.\n\nuint8 level";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/msg/SetLevelRequest";
    const TYPE_HASH: &'static str =
        "b3b8631db20cd753826003e821b752e0cb965e80cceec8b67c94cc5076a11b74";
}
impl SetLevelRequest {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
