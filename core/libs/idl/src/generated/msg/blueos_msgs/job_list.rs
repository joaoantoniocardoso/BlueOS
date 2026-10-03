#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JobList {
    pub jobs: Vec<crate::msg::blueos_msgs::JobStatus>,
}
impl CdrStruct for JobList {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            jobs: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(<crate::msg::blueos_msgs::JobStatus>::cdr_decode_fields(
                        reader,
                    )?);
                }
                Ok(values)
            })?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u32(self.jobs.len() as u32)?;
        for element in self.jobs.iter() {
            <crate::msg::blueos_msgs::JobStatus>::cdr_encode_fields(element, writer)?;
        }
        Ok(())
    }
}
impl Message for JobList {
    const SCHEMA: &'static str = "# blueos_msgs/msg/JobList\n# Snapshot published on blueos/v1/<service>/jobs.\n\nblueos_msgs/JobStatus[] jobs\n================================================================================\nMSG: blueos_msgs/JobStatus\n# blueos_msgs/msg/JobStatus\n# One Job in the jobs State (D-12, D-36). STATUS_ values 0 to 6 are those of ROS 2 action_msgs/GoalStatus; a ROS 2\n# client sees the two waiting statuses as STATUS_ACCEPTED, and STATUS_PAUSED as STATUS_EXECUTING (D-38).\n\nuint8 STATUS_UNKNOWN=0\nuint8 STATUS_ACCEPTED=1\nuint8 STATUS_EXECUTING=2\nuint8 STATUS_CANCELING=3\nuint8 STATUS_SUCCEEDED=4\nuint8 STATUS_CANCELED=5\nuint8 STATUS_ABORTED=6\nuint8 STATUS_WAITING_FOR_PERMISSION=7\nuint8 STATUS_WAITING_FOR_RESOURCE=8\nuint8 STATUS_PAUSED=9\n\n# The UUID the client generated for the Job, as text.\nstring job_id\n# The Job type, the name of its submit endpoint.\nstring job_type\nuint8 status\n# Why the Job was canceled or aborted, when the Kernel ended it.\nstring reason";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/JobList";
    const TYPE_HASH: &'static str =
        "d98875249e005b9cb94c99cb57b5b6c60cfd364bcbc3935493bcec3accd11f1e";
}
impl JobList {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
