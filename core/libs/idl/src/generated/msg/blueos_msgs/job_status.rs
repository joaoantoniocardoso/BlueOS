#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::string::String;
use serde::{Deserialize, Serialize};
pub mod constants_job_status {
    pub const STATUS_QUEUED: u8 = 0u8;
    pub const STATUS_RUNNING: u8 = 1u8;
    pub const STATUS_CANCELLING: u8 = 2u8;
    pub const STATUS_SUCCEEDED: u8 = 3u8;
    pub const STATUS_FAILED: u8 = 4u8;
    pub const STATUS_CANCELLED: u8 = 5u8;
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JobStatus {
    pub job_id: u64,
    pub parent_job_id: u64,
    pub status: u8,
    pub name: String,
}
impl CdrStruct for JobStatus {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            job_id: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            parent_job_id: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            status: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u8()?
            },
            name: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u64(self.job_id)?;
        writer.write_u64(self.parent_job_id)?;
        writer.write_u8(self.status)?;
        writer.write_string(self.name.as_str())?;
        Ok(())
    }
}
impl Message for JobStatus {
    const SCHEMA: &'static str = "# blueos_msgs/msg/JobStatus\n# One job entry; status values mirror blueos_jobs (D-12).\n\nuint8 STATUS_QUEUED=0\nuint8 STATUS_RUNNING=1\nuint8 STATUS_CANCELLING=2\nuint8 STATUS_SUCCEEDED=3\nuint8 STATUS_FAILED=4\nuint8 STATUS_CANCELLED=5\n\nuint64 job_id\nuint64 parent_job_id\nuint8 status\nstring name";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/JobStatus";
    const TYPE_HASH: &'static str =
        "eb55529f90cb8ed7d053a6c11ac965ae1b20f100071072431f3a275196ba197b";
}
impl JobStatus {
    pub const KNOWN_FIELD_COUNT: usize = 4usize;
}
