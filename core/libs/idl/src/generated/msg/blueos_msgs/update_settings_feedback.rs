#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateSettingsFeedback {}
impl CdrStruct for UpdateSettingsFeedback {
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
impl Message for UpdateSettingsFeedback {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_msgs/action/UpdateSettings_Feedback";
    const TYPE_HASH: &'static str =
        "8845d6d828c0f3bd1ffaffd545d26656b5ce229e3d3bf7af74bcb47cf48f0d86";
}
impl UpdateSettingsFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
