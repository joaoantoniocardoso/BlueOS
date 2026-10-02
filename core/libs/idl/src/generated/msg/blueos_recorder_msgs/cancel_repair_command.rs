#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CancelRepairCommand {
    pub path: String,
}
impl CdrStruct for CancelRepairCommand {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        Ok(())
    }
}
impl Message for CancelRepairCommand {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/CancelRepairCommand\n# Stops a running repair; the recording is left exactly as it was.\n\nstring path";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/CancelRepairCommand";
    const TYPE_HASH: &'static str =
        "abb5800cdb79877919b5977188c7da80253698471975eac5c1431ef7fc07fa4f";
}
impl CancelRepairCommand {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
