#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JobStatusStatus {
    #[default]
    StatusUnknown,
    Accepted,
    Executing,
    Canceling,
    Succeeded,
    Canceled,
    Aborted,
    WaitingForPermission,
    WaitingForResource,
    Paused,
    Unknown(u8),
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JobStatus {
    pub job_id: String,
    pub job_type: String,
    pub status: JobStatusStatus,
    pub reason: String,
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
impl JobStatusStatus {
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0u8 => Self::StatusUnknown,
            1u8 => Self::Accepted,
            2u8 => Self::Executing,
            3u8 => Self::Canceling,
            4u8 => Self::Succeeded,
            5u8 => Self::Canceled,
            6u8 => Self::Aborted,
            7u8 => Self::WaitingForPermission,
            8u8 => Self::WaitingForResource,
            9u8 => Self::Paused,
            raw => Self::Unknown(raw),
        }
    }
    pub fn as_raw(self) -> u8 {
        match self {
            Self::StatusUnknown => 0u8,
            Self::Accepted => 1u8,
            Self::Executing => 2u8,
            Self::Canceling => 3u8,
            Self::Succeeded => 4u8,
            Self::Canceled => 5u8,
            Self::Aborted => 6u8,
            Self::WaitingForPermission => 7u8,
            Self::WaitingForResource => 8u8,
            Self::Paused => 9u8,
            Self::Unknown(raw) => raw,
        }
    }
}
impl CdrStruct for JobStatus {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            job_id: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            job_type: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            status: if reader.is_exhausted() {
                <JobStatusStatus>::default()
            } else {
                JobStatusStatus::from_raw(reader.read_u8()?)
            },
            reason: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.job_id.as_str())?;
        writer.write_string(self.job_type.as_str())?;
        writer.write_u8(self.status.as_raw())?;
        writer.write_string(self.reason.as_str())?;
        Ok(())
    }
}
impl Message for JobStatus {
    const SCHEMA: &'static str = "# blueos_msgs/msg/JobStatus\n# One Job in the jobs State (D-12, D-36). STATUS_ values 0 to 6 are those of ROS 2 action_msgs/GoalStatus; a ROS 2\n# client sees the two waiting statuses as STATUS_ACCEPTED, and STATUS_PAUSED as STATUS_EXECUTING (D-38).\n\nuint8 STATUS_UNKNOWN=0\nuint8 STATUS_ACCEPTED=1\nuint8 STATUS_EXECUTING=2\nuint8 STATUS_CANCELING=3\nuint8 STATUS_SUCCEEDED=4\nuint8 STATUS_CANCELED=5\nuint8 STATUS_ABORTED=6\nuint8 STATUS_WAITING_FOR_PERMISSION=7\nuint8 STATUS_WAITING_FOR_RESOURCE=8\nuint8 STATUS_PAUSED=9\n\n# The UUID the client generated for the Job, as text.\nstring job_id\n# The Job type, the name of its submit endpoint.\nstring job_type\nuint8 status\n# Why the Job was canceled or aborted, when the Kernel ended it.\nstring reason";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/JobStatus";
    const TYPE_HASH: &'static str =
        "e56d38f8395c58c9a6519b065c940ea01b574dfb18d0958bc5f78517b97b1917";
}
impl JobStatus {
    pub const KNOWN_FIELD_COUNT: usize = 4usize;
}
