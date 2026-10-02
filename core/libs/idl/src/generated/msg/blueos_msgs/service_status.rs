#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServiceStatusStatus {
    #[default]
    StatusUnknown,
    Starting,
    Ready,
    Degraded,
    Stopping,
    Unknown(u8),
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub status: ServiceStatusStatus,
    pub detail: String,
}
impl serde::Serialize for ServiceStatusStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        <u8>::serialize(&self.as_raw(), serializer)
    }
}
impl<'de> serde::Deserialize<'de> for ServiceStatusStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from_raw(<u8>::deserialize(deserializer)?))
    }
}
impl ServiceStatusStatus {
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0u8 => Self::StatusUnknown,
            1u8 => Self::Starting,
            2u8 => Self::Ready,
            3u8 => Self::Degraded,
            4u8 => Self::Stopping,
            raw => Self::Unknown(raw),
        }
    }
    pub fn as_raw(self) -> u8 {
        match self {
            Self::StatusUnknown => 0u8,
            Self::Starting => 1u8,
            Self::Ready => 2u8,
            Self::Degraded => 3u8,
            Self::Stopping => 4u8,
            Self::Unknown(raw) => raw,
        }
    }
}
impl CdrStruct for ServiceStatus {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            status: if reader.is_exhausted() {
                <ServiceStatusStatus>::default()
            } else {
                ServiceStatusStatus::from_raw(reader.read_u8()?)
            },
            detail: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(self.status.as_raw())?;
        writer.write_string(self.detail.as_str())?;
        Ok(())
    }
}
impl Message for ServiceStatus {
    const SCHEMA: &'static str = "# blueos_msgs/msg/ServiceStatus\n# High-level service health on the status state key (D-12).\n\nuint8 STATUS_UNKNOWN=0\nuint8 STATUS_STARTING=1\nuint8 STATUS_READY=2\nuint8 STATUS_DEGRADED=3\nuint8 STATUS_STOPPING=4\n\nuint8 status\nstring detail";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/ServiceStatus";
    const TYPE_HASH: &'static str =
        "d7acf6755b7629bf8da3f41e1d2734b9dcb27bf7dc0d25d1c1a28e09f33dc875";
}
impl ServiceStatus {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
