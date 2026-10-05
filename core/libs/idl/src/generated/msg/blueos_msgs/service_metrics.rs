#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub counters: Vec<crate::msg::blueos_msgs::MetricCounter>,
    pub gauges: Vec<crate::msg::blueos_msgs::MetricGauge>,
    pub histograms: Vec<crate::msg::blueos_msgs::MetricHistogram>,
}
impl CdrStruct for ServiceMetrics {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            counters: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(<crate::msg::blueos_msgs::MetricCounter>::cdr_decode_fields(
                        reader,
                    )?);
                }
                Ok(values)
            })?,
            gauges: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(<crate::msg::blueos_msgs::MetricGauge>::cdr_decode_fields(
                        reader,
                    )?);
                }
                Ok(values)
            })?,
            histograms: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(
                        <crate::msg::blueos_msgs::MetricHistogram>::cdr_decode_fields(reader)?,
                    );
                }
                Ok(values)
            })?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u32(self.counters.len() as u32)?;
        for element in self.counters.iter() {
            <crate::msg::blueos_msgs::MetricCounter>::cdr_encode_fields(element, writer)?;
        }
        writer.write_u32(self.gauges.len() as u32)?;
        for element in self.gauges.iter() {
            <crate::msg::blueos_msgs::MetricGauge>::cdr_encode_fields(element, writer)?;
        }
        writer.write_u32(self.histograms.len() as u32)?;
        for element in self.histograms.iter() {
            <crate::msg::blueos_msgs::MetricHistogram>::cdr_encode_fields(element, writer)?;
        }
        Ok(())
    }
}
impl Message for ServiceMetrics {
    const SCHEMA: &'static str = "# blueos_msgs/msg/ServiceMetrics\n# State published on blueos/v1/<service>/state/metrics (D-12, D-35): every counter, gauge and histogram that the\n# Kernel, the Tasks and the adapters of the service recorded, each list sorted by name and then by labels. It is\n# published at most once per second, and only when a value changed.\n\nblueos_msgs/MetricCounter[] counters\nblueos_msgs/MetricGauge[] gauges\nblueos_msgs/MetricHistogram[] histograms\n================================================================================\nMSG: blueos_msgs/MetricLabel\n# blueos_msgs/msg/MetricLabel\n# One label of a metric in ServiceMetrics (D-35), such as the Task a restart counter counts.\n\nstring name\nstring value\n================================================================================\nMSG: blueos_msgs/MetricCounter\n# blueos_msgs/msg/MetricCounter\n# A counter in ServiceMetrics (D-35): a total that only grows while the service runs.\n\nstring name\nblueos_msgs/MetricLabel[] labels\nuint64 value\n================================================================================\nMSG: blueos_msgs/MetricGauge\n# blueos_msgs/msg/MetricGauge\n# A gauge in ServiceMetrics (D-35): the last value the service set, such as how many Commands wait in the Inbox.\n\nstring name\nblueos_msgs/MetricLabel[] labels\nfloat64 value\n================================================================================\nMSG: blueos_msgs/MetricHistogram\n# blueos_msgs/msg/MetricHistogram\n# A histogram in ServiceMetrics (D-35), such as how long the Inbox took to apply each Command: how many values the\n# service recorded, their sum, and how many fell in each bucket.\n\nstring name\nblueos_msgs/MetricLabel[] labels\nuint64 count\nfloat64 sum\n# The upper bound of each bucket, ascending. A bucket holds the values above the bound before it, up to its own.\nfloat64[] bucket_bounds\n# How many values fell in each bucket of bucket_bounds, plus one last entry for the values above the last bound.\nuint64[] bucket_counts";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/ServiceMetrics";
    const TYPE_HASH: &'static str =
        "152637ca9b0c85b40aa3aa359f5dac7aad3a0ba937ede7ad891c724178ccce37";
}
impl ServiceMetrics {
    pub const KNOWN_FIELD_COUNT: usize = 3usize;
}
