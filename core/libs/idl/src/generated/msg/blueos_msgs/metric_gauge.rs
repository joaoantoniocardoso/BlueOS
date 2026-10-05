#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MetricGauge {
    pub name: String,
    pub labels: Vec<crate::msg::blueos_msgs::MetricLabel>,
    pub value: f64,
}
impl CdrStruct for MetricGauge {
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
            value: reader.read_or_default(|reader| reader.read_f64())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.name.as_str())?;
        writer.write_u32(self.labels.len() as u32)?;
        for element in self.labels.iter() {
            <crate::msg::blueos_msgs::MetricLabel>::cdr_encode_fields(element, writer)?;
        }
        writer.write_f64(self.value)?;
        Ok(())
    }
}
impl Message for MetricGauge {
    const SCHEMA: &'static str = "# blueos_msgs/msg/MetricGauge\n# A gauge in ServiceMetrics (D-35): the last value the service set, such as how many Commands wait in the Inbox.\n\nstring name\nblueos_msgs/MetricLabel[] labels\nfloat64 value\n================================================================================\nMSG: blueos_msgs/MetricLabel\n# blueos_msgs/msg/MetricLabel\n# One label of a metric in ServiceMetrics (D-35), such as the Task a restart counter counts.\n\nstring name\nstring value";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/MetricGauge";
    const TYPE_HASH: &'static str =
        "ea0aac4b769c840cb949fa4854dabb3948f5a0d135ccfd058b26604d5fe66640";
}
impl MetricGauge {
    pub const KNOWN_FIELD_COUNT: usize = 3usize;
}
