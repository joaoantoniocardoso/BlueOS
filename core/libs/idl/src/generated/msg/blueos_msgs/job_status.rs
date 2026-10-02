#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::string::String;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JobStatusStatus {
    #[default]
    Queued,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
    Unknown(u8),
}
impl JobStatusStatus {
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0u8 => Self::Queued,
            1u8 => Self::Running,
            2u8 => Self::Cancelling,
            3u8 => Self::Succeeded,
            4u8 => Self::Failed,
            5u8 => Self::Cancelled,
            raw => Self::Unknown(raw),
        }
    }
    pub fn as_raw(self) -> u8 {
        match self {
            Self::Queued => 0u8,
            Self::Running => 1u8,
            Self::Cancelling => 2u8,
            Self::Succeeded => 3u8,
            Self::Failed => 4u8,
            Self::Cancelled => 5u8,
            Self::Unknown(raw) => raw,
        }
    }
}
impl serde::Serialize for JobStatusStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        <u8>::serialize(&self.as_raw(), serializer)
    }
}
impl<'de> serde::Deserialize<'de> for JobStatusStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from_raw(<u8>::deserialize(deserializer)?))
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JobStatus {
    pub job_id: u64,
    pub parent_job_id: u64,
    pub status: JobStatusStatus,
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
                <JobStatusStatus>::default()
            } else {
                JobStatusStatus::from_raw(reader.read_u8()?)
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
        writer.write_u8(self.status.as_raw())?;
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
