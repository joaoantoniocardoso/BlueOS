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
            jobs: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(<crate::msg::blueos_msgs::JobStatus>::cdr_decode_fields(
                            reader,
                        )?);
                    }
                    values
                }
            },
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
    const SCHEMA: &'static str = "# blueos_msgs/msg/JobList\n# Snapshot published on blueos/v1/<service>/jobs.\n\nblueos_msgs/JobStatus[] jobs\n================================================================================\nMSG: blueos_msgs/JobStatus\n# blueos_msgs/msg/JobStatus\n# One job entry; status values mirror blueos_jobs (D-12).\n\nuint8 STATUS_QUEUED=0\nuint8 STATUS_RUNNING=1\nuint8 STATUS_CANCELLING=2\nuint8 STATUS_SUCCEEDED=3\nuint8 STATUS_FAILED=4\nuint8 STATUS_CANCELLED=5\nuint8 STATUS_INTERRUPTED=6\n\nuint64 job_id\nuint64 parent_job_id\nuint8 status\nstring name";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/JobList";
    const TYPE_HASH: &'static str =
        "d98875249e005b9cb94c99cb57b5b6c60cfd364bcbc3935493bcec3accd11f1e";
}
impl JobList {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
