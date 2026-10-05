#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EndpointInfo {
    pub kind: String,
    pub name: String,
    pub key: String,
    pub interface_type: String,
    pub schema: String,
}
impl CdrStruct for EndpointInfo {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            kind: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            name: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            key: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            interface_type: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            schema: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.kind.as_str())?;
        writer.write_string(self.name.as_str())?;
        writer.write_string(self.key.as_str())?;
        writer.write_string(self.interface_type.as_str())?;
        writer.write_string(self.schema.as_str())?;
        Ok(())
    }
}
impl Message for EndpointInfo {
    const SCHEMA: &'static str = "# blueos_msgs/msg/EndpointInfo\n# One key a service serves, listed in ServiceInfo.endpoints so clients can discover the API (D-12, D-24).\n\n# One of: job, query, state, event.\nstring kind\nstring name\nstring key\n# A .action for a job, a .srv for a query, a .msg for a state or an event.\nstring interface_type\n# The schema text of interface_type. That of a .action or a .srv lists every part. That of a Job's feedback or\n# result, a blueos_msgs/JobFeedbackList or blueos_msgs/JobResult, then lists the action part its bytes carry.\nstring schema";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/EndpointInfo";
    const TYPE_HASH: &'static str =
        "251a0a01220bdc22c550c09b88a34d3c54c3aa77cd34deab5e0dd90c3d52cfc2";
}
impl EndpointInfo {
    pub const KNOWN_FIELD_COUNT: usize = 5usize;
}
