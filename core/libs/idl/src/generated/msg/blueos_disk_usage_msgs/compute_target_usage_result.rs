#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ComputeTargetUsageResult {
    pub usage: crate::msg::blueos_disk_usage_msgs::DiskUsage,
}
impl CdrStruct for ComputeTargetUsageResult {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            usage: reader.read_or_default(|reader| {
                <crate::msg::blueos_disk_usage_msgs::DiskUsage>::cdr_decode_fields(reader)
            })?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        <crate::msg::blueos_disk_usage_msgs::DiskUsage>::cdr_encode_fields(&self.usage, writer)?;
        Ok(())
    }
}
impl Message for ComputeTargetUsageResult {
    const SCHEMA: &'static str = "# Job result: complete scan.\nblueos_disk_usage_msgs/DiskUsage usage\n================================================================================\nMSG: blueos_disk_usage_msgs/DiskNode\n# blueos_disk_usage_msgs/msg/DiskNode\n# One file or directory in a usage tree.\n\nstring name\nstring path\nuint64 size_bytes\nbool is_dir\n================================================================================\nMSG: builtin_interfaces/Time\n# This message communicates ROS Time defined here:\n# https://design.ros2.org/articles/clock_and_time.html\n\n# The seconds component, valid over all int32 values.\nint32 sec\n\n# The nanoseconds component, valid in the range [0, 1e9).\nuint32 nanosec\n================================================================================\nMSG: blueos_disk_usage_msgs/DiskUsage\n# blueos_disk_usage_msgs/msg/DiskUsage\n# Result of a usage scan: flat nodes list plus the parameters that produced it.\n\nblueos_disk_usage_msgs/DiskNode[] nodes\nbuiltin_interfaces/Time generated_at\nuint8 depth\nbool include_files\nuint64 min_size_bytes";
    const SCHEMA_NAME: &'static str = "blueos_disk_usage_msgs/action/ComputeTargetUsage_Result";
    const TYPE_HASH: &'static str =
        "4cd47f42c1bbdfc486c2b96c2a3e69ef578cd7393a9f020d6d27e876d89d3bba";
}
impl ComputeTargetUsageResult {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
