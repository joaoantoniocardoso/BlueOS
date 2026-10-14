#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ComputeTargetUsageFeedback {
    pub du_lines_parsed: u64,
    pub nodes_added: Vec<crate::msg::blueos_disk_usage_msgs::DiskNode>,
}
impl CdrStruct for ComputeTargetUsageFeedback {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            du_lines_parsed: reader.read_or_default(|reader| reader.read_u64())?,
            nodes_added: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(
                        <crate::msg::blueos_disk_usage_msgs::DiskNode>::cdr_decode_fields(reader)?,
                    );
                }
                Ok(values)
            })?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u64(self.du_lines_parsed)?;
        writer.write_u32(self.nodes_added.len() as u32)?;
        for element in self.nodes_added.iter() {
            <crate::msg::blueos_disk_usage_msgs::DiskNode>::cdr_encode_fields(element, writer)?;
        }
        Ok(())
    }
}
impl Message for ComputeTargetUsageFeedback {
    const SCHEMA: &'static str = "# Feedback: progress while scanning.\nuint64 du_lines_parsed\nblueos_disk_usage_msgs/DiskNode[] nodes_added\n================================================================================\nMSG: blueos_disk_usage_msgs/DiskNode\n# blueos_disk_usage_msgs/msg/DiskNode\n# One file or directory in a usage tree.\n\nstring name\nstring path\nuint64 size_bytes\nbool is_dir";
    const SCHEMA_NAME: &'static str = "blueos_disk_usage_msgs/action/ComputeTargetUsage_Feedback";
    const TYPE_HASH: &'static str =
        "916e1efbb3fe7892a4dd982596ae4a690a07cb273f3ca86d079690763c29db8f";
}
impl ComputeTargetUsageFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
