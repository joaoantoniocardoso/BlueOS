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
    pub request_schema: String,
    pub response_schema: String,
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
            request_schema: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            response_schema: if reader.is_exhausted() {
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
        writer.write_string(self.request_schema.as_str())?;
        writer.write_string(self.response_schema.as_str())?;
        Ok(())
    }
}
impl Message for EndpointInfo {
    const SCHEMA: &'static str = "# blueos_msgs/msg/EndpointInfo\n# One key a service serves, listed in ServiceInfo.endpoints so clients can discover the API (D-12, D-24).\n\n# One of: command, query, io_query, state, event.\nstring kind\nstring name\nstring key\n# Schema of the payload the client sends (command, query, io_query). Empty when the endpoint takes none.\nstring request_schema\n# Schema of the reply (command: CommandAck; query, io_query) or of the published sample (state, event).\nstring response_schema";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/EndpointInfo";
    const TYPE_HASH: &'static str =
        "76ab474b0e3bb9c16e3d681648503ea768ebfe7a0841db5cb271cd6a8fb62128";
}
impl EndpointInfo {
    pub const KNOWN_FIELD_COUNT: usize = 5usize;
}
