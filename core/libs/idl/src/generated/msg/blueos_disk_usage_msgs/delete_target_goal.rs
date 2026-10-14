#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteTargetGoal {
    pub target: String,
}
impl CdrStruct for DeleteTargetGoal {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            target: reader.read_or_default(|reader| reader.read_string())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.target.as_str())?;
        Ok(())
    }
}
impl Message for DeleteTargetGoal {
    const SCHEMA: &'static str = "# blueos_disk_usage_msgs/action/DeleteTarget\n# The Job type on blueos/v1/disk_usage/command/DeleteTarget. Rejected when the target is protected,\n# not found, outside root, is already being deleted by another Job.\n# deleted. The Job succeeds once the target is gone.\n\nstring target";
    const SCHEMA_NAME: &'static str = "blueos_disk_usage_msgs/action/DeleteTarget_Goal";
    const TYPE_HASH: &'static str =
        "c302176134c38e7a28fa04235c4f42d625fea83d188250c98db477593f5283f8";
}
impl DeleteTargetGoal {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
