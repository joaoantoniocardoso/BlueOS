#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MeasureResponse {
    pub level: f32,
    pub progress: crate::msg::fixture_msgs::Progress,
}
impl CdrStruct for MeasureResponse {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            level: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_f32()?
            },
            progress: if reader.is_exhausted() {
                <crate::msg::fixture_msgs::Progress>::default()
            } else {
                <crate::msg::fixture_msgs::Progress>::cdr_decode_fields(reader)?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_f32(self.level)?;
        <crate::msg::fixture_msgs::Progress>::cdr_encode_fields(&self.progress, writer)?;
        Ok(())
    }
}
impl Message for MeasureResponse {
    const SCHEMA: &'static str = "float32 level\nProgress progress\n================================================================================\nMSG: fixture_msgs/Progress\n# fixture_msgs/msg/Progress\nuint64 done\nuint64 total";
    const SCHEMA_NAME: &'static str = "fixture_msgs/srv/Measure_Response";
    const TYPE_HASH: &'static str =
        "24a054ecab7e9d893010e3ff94fa117b272d1e7d734481817d10f603b69e5424";
}
impl MeasureResponse {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
