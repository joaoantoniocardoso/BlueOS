#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
    pub build: String,
    pub capabilities: Vec<String>,
    pub endpoints: Vec<crate::msg::blueos_msgs::EndpointInfo>,
}
impl CdrStruct for ServiceInfo {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            name: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            version: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            build: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            capabilities: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(reader.read_string()?);
                    }
                    values
                }
            },
            endpoints: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(<crate::msg::blueos_msgs::EndpointInfo>::cdr_decode_fields(
                            reader,
                        )?);
                    }
                    values
                }
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.name.as_str())?;
        writer.write_string(self.version.as_str())?;
        writer.write_string(self.build.as_str())?;
        writer.write_u32(self.capabilities.len() as u32)?;
        for element in self.capabilities.iter() {
            writer.write_string(element.as_str())?;
        }
        writer.write_u32(self.endpoints.len() as u32)?;
        for element in self.endpoints.iter() {
            <crate::msg::blueos_msgs::EndpointInfo>::cdr_encode_fields(element, writer)?;
        }
        Ok(())
    }
}
impl Message for ServiceInfo {
    const SCHEMA: &'static str = "# blueos_msgs/msg/ServiceInfo\n# Metadata exposed on blueos/v1/<service>/info (D-12).\n\nstring name\nstring version\nstring build\nstring[] capabilities\nblueos_msgs/EndpointInfo[] endpoints\n================================================================================\nMSG: blueos_msgs/EndpointInfo\n# blueos_msgs/msg/EndpointInfo\n# One key a service serves, listed in ServiceInfo.endpoints so clients can discover the API (D-12, D-24).\n\n# One of: command, query, io_query, state, event.\nstring kind\nstring name\nstring key\n# Schema of the payload the client sends (command, query, io_query). Empty when the endpoint takes none.\nstring request_schema\n# Schema of the reply (command: CommandAck; query, io_query) or of the published sample (state, event).\nstring response_schema";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/ServiceInfo";
    const TYPE_HASH: &'static str =
        "d3beeba44fd29bcab28c5fd30cb70effbcf27eb9c274e1a22b16ec62d3a3e937";
}
impl ServiceInfo {
    pub const KNOWN_FIELD_COUNT: usize = 5usize;
}
