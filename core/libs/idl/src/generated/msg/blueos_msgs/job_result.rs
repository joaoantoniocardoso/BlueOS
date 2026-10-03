#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JobResult {
    pub job: crate::msg::blueos_msgs::JobStatus,
    pub result: Vec<u8>,
}
impl CdrStruct for JobResult {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            job: if reader.is_exhausted() {
                <crate::msg::blueos_msgs::JobStatus>::default()
            } else {
                <crate::msg::blueos_msgs::JobStatus>::cdr_decode_fields(reader)?
            },
            result: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(reader.read_u8()?);
                    }
                    values
                }
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        <crate::msg::blueos_msgs::JobStatus>::cdr_encode_fields(&self.job, writer)?;
        writer.write_u32(self.result.len() as u32)?;
        for element in self.result.iter() {
            writer.write_u8(*element)?;
        }
        Ok(())
    }
}
impl Message for JobResult {
    const SCHEMA: &'static str = "# blueos_msgs/msg/JobResult\n# Event published on blueos/v1/<service>/jobs/<JobType>/result when a Job ends (D-12, D-36), like the result of a ROS 2\n# action.\n\n# The Job as it ended: Succeeded, Canceled or Aborted, with its reason.\nblueos_msgs/JobStatus job\n# The Job type's Job result message, CDR with its encapsulation header. Empty when the Job type declares none.\nuint8[] result\n================================================================================\nMSG: blueos_msgs/JobStatus\n# blueos_msgs/msg/JobStatus\n# One Job in the jobs State (D-12, D-36). STATUS_ values 0 to 6 are those of ROS 2 action_msgs/GoalStatus; a ROS 2\n# client sees the two waiting statuses as STATUS_ACCEPTED, and STATUS_PAUSED as STATUS_EXECUTING (D-38).\n\nuint8 STATUS_UNKNOWN=0\nuint8 STATUS_ACCEPTED=1\nuint8 STATUS_EXECUTING=2\nuint8 STATUS_CANCELING=3\nuint8 STATUS_SUCCEEDED=4\nuint8 STATUS_CANCELED=5\nuint8 STATUS_ABORTED=6\nuint8 STATUS_WAITING_FOR_PERMISSION=7\nuint8 STATUS_WAITING_FOR_RESOURCE=8\nuint8 STATUS_PAUSED=9\n\n# The UUID the client generated for the Job, as text.\nstring job_id\n# The Job type, the name of its submit endpoint.\nstring job_type\nuint8 status\n# Why the Job was canceled or aborted, when the Kernel ended it.\nstring reason";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/JobResult";
    const TYPE_HASH: &'static str =
        "e9d5178c913dc4b278216524e227f2705ffa220e82f172bfc30889d6134a59d7";
}
impl JobResult {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
