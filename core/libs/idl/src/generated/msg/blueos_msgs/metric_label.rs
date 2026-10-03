#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MetricLabel {
    pub name: String,
    pub value: String,
}
impl CdrStruct for MetricLabel {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            name: reader.read_or_default(|reader| reader.read_string())?,
            value: reader.read_or_default(|reader| reader.read_string())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.name.as_str())?;
        writer.write_string(self.value.as_str())?;
        Ok(())
    }
}
impl Message for MetricLabel {
    const SCHEMA: &'static str = "# blueos_msgs/msg/MetricLabel\n# One label of a metric in ServiceMetrics (D-35), such as the Task a restart counter counts.\n\nstring name\nstring value";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/MetricLabel";
    const TYPE_HASH: &'static str =
        "9e368be59d443edc527539b5df10f2407869fcbe7084a7794e2d565ac0df84ae";
}
impl MetricLabel {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
