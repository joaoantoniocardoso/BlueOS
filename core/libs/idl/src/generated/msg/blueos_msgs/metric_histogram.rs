#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MetricHistogram {
    pub name: String,
    pub labels: Vec<crate::msg::blueos_msgs::MetricLabel>,
    pub count: u64,
    pub sum: f64,
    pub bucket_bounds: Vec<f64>,
    pub bucket_counts: Vec<u64>,
}
impl CdrStruct for MetricHistogram {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            name: reader.read_or_default(|reader| reader.read_string())?,
            labels: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(<crate::msg::blueos_msgs::MetricLabel>::cdr_decode_fields(
                        reader,
                    )?);
                }
                Ok(values)
            })?,
            count: reader.read_or_default(|reader| reader.read_u64())?,
            sum: reader.read_or_default(|reader| reader.read_f64())?,
            bucket_bounds: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(reader.read_f64()?);
                }
                Ok(values)
            })?,
            bucket_counts: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(reader.read_u64()?);
                }
                Ok(values)
            })?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.name.as_str())?;
        writer.write_u32(self.labels.len() as u32)?;
        for element in self.labels.iter() {
            <crate::msg::blueos_msgs::MetricLabel>::cdr_encode_fields(element, writer)?;
        }
        writer.write_u64(self.count)?;
        writer.write_f64(self.sum)?;
        writer.write_u32(self.bucket_bounds.len() as u32)?;
        for element in self.bucket_bounds.iter() {
            writer.write_f64(*element)?;
        }
        writer.write_u32(self.bucket_counts.len() as u32)?;
        for element in self.bucket_counts.iter() {
            writer.write_u64(*element)?;
        }
        Ok(())
    }
}
impl Message for MetricHistogram {
    const SCHEMA: &'static str = "# blueos_msgs/msg/MetricHistogram\n# A histogram in ServiceMetrics (D-35), such as how long the Inbox took to apply each Command: how many values the\n# service recorded, their sum, and how many fell in each bucket.\n\nstring name\nblueos_msgs/MetricLabel[] labels\nuint64 count\nfloat64 sum\n# The upper bound of each bucket, ascending. A bucket holds the values above the bound before it, up to its own.\nfloat64[] bucket_bounds\n# How many values fell in each bucket of bucket_bounds, plus one last entry for the values above the last bound.\nuint64[] bucket_counts\n================================================================================\nMSG: blueos_msgs/MetricLabel\n# blueos_msgs/msg/MetricLabel\n# One label of a metric in ServiceMetrics (D-35), such as the Task a restart counter counts.\n\nstring name\nstring value";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/MetricHistogram";
    const TYPE_HASH: &'static str =
        "525e82352066eadd4af7f4cd4718c5a090637e2055b2e4d18cf719b7dbd9ea69";
}
impl MetricHistogram {
    pub const KNOWN_FIELD_COUNT: usize = 6usize;
}
