#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FillGoal {
    pub level: f32,
    pub rate: f32,
}
impl CdrStruct for FillGoal {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            level: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_f32()?
            },
            rate: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_f32()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_f32(self.level)?;
        writer.write_f32(self.rate)?;
        Ok(())
    }
}
impl Message for FillGoal {
    const SCHEMA: &'static str = "# fixture_msgs/action/Fill\nfloat32 level\nfloat32 rate";
    const SCHEMA_NAME: &'static str = "fixture_msgs/action/Fill_Goal";
    const TYPE_HASH: &'static str =
        "b14106c8d4423f47033c5923886a2dc83db83ac470bc993f16b2d4173e2d0d63";
}
impl FillGoal {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
