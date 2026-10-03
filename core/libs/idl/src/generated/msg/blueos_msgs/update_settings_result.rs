#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateSettingsResult {}
impl CdrStruct for UpdateSettingsResult {
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
impl Message for UpdateSettingsResult {
    const SCHEMA: &'static str = "";
    const SCHEMA_NAME: &'static str = "blueos_msgs/action/UpdateSettings_Result";
    const TYPE_HASH: &'static str =
        "d41261fe75a375b194e25bd9ab8227dcf6d4dec1ad9870d68ffb1b03c3fe05f9";
}
impl UpdateSettingsResult {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
