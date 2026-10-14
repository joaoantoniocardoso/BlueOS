#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteTargetResult {
    pub target: String,
}
impl CdrStruct for DeleteTargetResult {
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
impl Message for DeleteTargetResult {
    const SCHEMA: &'static str = "# Job result: the target the delete Job executed. Why it failed is the reason of the Job.\nstring target";
    const SCHEMA_NAME: &'static str = "blueos_disk_usage_msgs/action/DeleteTarget_Result";
    const TYPE_HASH: &'static str =
        "bddfa8ceb54ad7d66cdcb53196b6642ae86f7b69a25a01619261f62c44e3c078";
}
impl DeleteTargetResult {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
