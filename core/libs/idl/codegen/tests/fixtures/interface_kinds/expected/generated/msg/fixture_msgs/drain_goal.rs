#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DrainGoal {}
impl CdrStruct for DrainGoal {
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
impl Message for DrainGoal {
    const SCHEMA: &'static str =
        "# fixture_msgs/action/Drain\n# An empty Goal: draining needs no input.";
    const SCHEMA_NAME: &'static str = "fixture_msgs/action/Drain_Goal";
    const TYPE_HASH: &'static str =
        "d0691f645363e65028d46c51fdaa2fcc8e921ebaec1fc0216ed5ae0384f15ad2";
}
impl DrainGoal {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
