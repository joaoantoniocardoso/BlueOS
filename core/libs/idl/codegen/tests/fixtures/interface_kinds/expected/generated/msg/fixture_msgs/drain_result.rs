#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DrainResult {
    pub drained: f32,
}
impl CdrStruct for DrainResult {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            drained: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_f32()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_f32(self.drained)?;
        Ok(())
    }
}
impl Message for DrainResult {
    const SCHEMA: &'static str = "float32 drained";
    const SCHEMA_NAME: &'static str = "fixture_msgs/action/Drain_Result";
    const TYPE_HASH: &'static str =
        "5b898146d3b861426ecf681360d85e7fa92b4d46dbb57942a0c4a4331e3eb0b4";
}
impl DrainResult {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
