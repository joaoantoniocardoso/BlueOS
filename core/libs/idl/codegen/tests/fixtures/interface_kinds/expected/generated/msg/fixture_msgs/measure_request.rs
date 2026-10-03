#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MeasureRequest {
    pub probe: String,
}
impl CdrStruct for MeasureRequest {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            probe: reader.read_or_default(|reader| reader.read_string())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.probe.as_str())?;
        Ok(())
    }
}
impl Message for MeasureRequest {
    const SCHEMA: &'static str = "# fixture_msgs/srv/Measure\nstring probe";
    const SCHEMA_NAME: &'static str = "fixture_msgs/srv/Measure_Request";
    const TYPE_HASH: &'static str =
        "ec5ec8f8bff29259e477ab199b7cac218527521b2acd68d4cd53d963d3d0f03c";
}
impl MeasureRequest {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
