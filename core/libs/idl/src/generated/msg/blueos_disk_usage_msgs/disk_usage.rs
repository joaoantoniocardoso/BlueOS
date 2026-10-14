#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DiskUsage {
    pub nodes: Vec<crate::msg::blueos_disk_usage_msgs::DiskNode>,
    pub generated_at: crate::msg::builtin_interfaces::Time,
    pub depth: u8,
    pub include_files: bool,
    pub min_size_bytes: u64,
}
impl CdrStruct for DiskUsage {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            nodes: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(
                        <crate::msg::blueos_disk_usage_msgs::DiskNode>::cdr_decode_fields(reader)?,
                    );
                }
                Ok(values)
            })?,
            generated_at: reader.read_or_default(|reader| {
                <crate::msg::builtin_interfaces::Time>::cdr_decode_fields(reader)
            })?,
            depth: reader.read_or_default(|reader| reader.read_u8())?,
            include_files: reader.read_or_default(|reader| reader.read_bool())?,
            min_size_bytes: reader.read_or_default(|reader| reader.read_u64())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u32(self.nodes.len() as u32)?;
        for element in self.nodes.iter() {
            <crate::msg::blueos_disk_usage_msgs::DiskNode>::cdr_encode_fields(element, writer)?;
        }
        <crate::msg::builtin_interfaces::Time>::cdr_encode_fields(&self.generated_at, writer)?;
        writer.write_u8(self.depth)?;
        writer.write_bool(self.include_files)?;
        writer.write_u64(self.min_size_bytes)?;
        Ok(())
    }
}
impl Message for DiskUsage {
    const SCHEMA: &'static str = "# blueos_disk_usage_msgs/msg/DiskUsage\n# Result of a usage scan: flat nodes list plus the parameters that produced it.\n\nblueos_disk_usage_msgs/DiskNode[] nodes\nbuiltin_interfaces/Time generated_at\nuint8 depth\nbool include_files\nuint64 min_size_bytes\n================================================================================\nMSG: blueos_disk_usage_msgs/DiskNode\n# blueos_disk_usage_msgs/msg/DiskNode\n# One file or directory in a usage tree.\n\nstring name\nstring path\nuint64 size_bytes\nbool is_dir\n================================================================================\nMSG: builtin_interfaces/Time\n# This message communicates ROS Time defined here:\n# https://design.ros2.org/articles/clock_and_time.html\n\n# The seconds component, valid over all int32 values.\nint32 sec\n\n# The nanoseconds component, valid in the range [0, 1e9).\nuint32 nanosec";
    const SCHEMA_NAME: &'static str = "blueos_disk_usage_msgs/msg/DiskUsage";
    const TYPE_HASH: &'static str =
        "c8f7afd21053e6cd3225c749d2472107139b35c8ef00d3009ac69257bd103abe";
}
impl DiskUsage {
    pub const KNOWN_FIELD_COUNT: usize = 5usize;
}
