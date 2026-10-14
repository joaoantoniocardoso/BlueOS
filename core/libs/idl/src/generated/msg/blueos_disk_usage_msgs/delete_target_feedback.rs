#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteTargetFeedback {
    pub entries_removed: u64,
    pub current_target: String,
}
impl CdrStruct for DeleteTargetFeedback {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            entries_removed: reader.read_or_default(|reader| reader.read_u64())?,
            current_target: reader.read_or_default(|reader| reader.read_string())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u64(self.entries_removed)?;
        writer.write_string(self.current_target.as_str())?;
        Ok(())
    }
}
impl Message for DeleteTargetFeedback {
    const SCHEMA: &'static str =
        "# Feedback: progress while deleting.\nuint64 entries_removed\nstring current_target";
    const SCHEMA_NAME: &'static str = "blueos_disk_usage_msgs/action/DeleteTarget_Feedback";
    const TYPE_HASH: &'static str =
        "5cc8fd6f70b198a5e2d31b6b8df8a9311b112bc63645b09ea06ee2acde4f497f";
}
impl DeleteTargetFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
