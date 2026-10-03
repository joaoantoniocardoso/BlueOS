#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FillResult {
    pub reached: bool,
}
impl CdrStruct for FillResult {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            reached: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_bool(self.reached)?;
        Ok(())
    }
}
impl Message for FillResult {
    const SCHEMA: &'static str = "bool reached";
    const SCHEMA_NAME: &'static str = "fixture_msgs/action/Fill_Result";
    const TYPE_HASH: &'static str =
        "9c6b14f76909002fb48bc06c594094749fc3ee4fede7e37f8de9d3a0744a52b4";
}
impl FillResult {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
