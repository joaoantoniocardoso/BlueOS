#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ComputeTargetUsageGoal {
    pub target: String,
    pub depth: u8,
    pub include_files: bool,
    pub min_size_bytes: u64,
}
impl CdrStruct for ComputeTargetUsageGoal {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            target: reader.read_or_default(|reader| reader.read_string())?,
            depth: reader.read_or_default(|reader| reader.read_u8())?,
            include_files: reader.read_or_default(|reader| reader.read_bool())?,
            min_size_bytes: reader.read_or_default(|reader| reader.read_u64())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.target.as_str())?;
        writer.write_u8(self.depth)?;
        writer.write_bool(self.include_files)?;
        writer.write_u64(self.min_size_bytes)?;
        Ok(())
    }
}
impl Message for ComputeTargetUsageGoal {
    const SCHEMA: &'static str = "# blueos_disk_usage_msgs/action/ComputeTargetUsage\n# The Job type on blueos/v1/disk_usage/command/ComputeTargetUsage: compute disk usage for a target folder recursively.\n\n# Target: the target (file or folder) to inspect.\nstring target\n# Depth: the maximum depth to inspect.\nuint8 depth\n# IncludeFiles: whether to include files in the output.\nbool include_files\n# MinSizeBytes: filter out entries smaller than this size (except for the root).\nuint64 min_size_bytes";
    const SCHEMA_NAME: &'static str = "blueos_disk_usage_msgs/action/ComputeTargetUsage_Goal";
    const TYPE_HASH: &'static str =
        "d4b35b983f2005d72822bb84a7ac9e70bd77664d71f3538e8215ca1513735af2";
}
impl ComputeTargetUsageGoal {
    pub const KNOWN_FIELD_COUNT: usize = 4usize;
}
