#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::string::String;
use serde::{Deserialize, Serialize};
pub mod constants_service_status {
    pub const STATUS_UNKNOWN: u8 = 0u8;
    pub const STATUS_STARTING: u8 = 1u8;
    pub const STATUS_READY: u8 = 2u8;
    pub const STATUS_DEGRADED: u8 = 3u8;
    pub const STATUS_STOPPING: u8 = 4u8;
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub status: u8,
    pub detail: String,
}
impl CdrStruct for ServiceStatus {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            status: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u8()?
            },
            detail: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(self.status)?;
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
