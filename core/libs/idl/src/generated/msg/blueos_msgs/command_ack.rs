#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommandAck {
    pub accepted: bool,
    pub job_id: u64,
    pub reason: String,
}
impl CdrStruct for CommandAck {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            accepted: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
            job_id: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            reason: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_bool(self.accepted)?;
        writer.write_u64(self.job_id)?;
        writer.write_string(self.reason.as_str())?;
        Ok(())
    }
}
impl Message for CommandAck {
    const SCHEMA: &'static str = "# blueos_msgs/msg/CommandAck\n# Reply to a Zenoh command query (D-10).\n\nbool accepted\nuint64 job_id\nstring reason";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/CommandAck";
    const TYPE_HASH: &'static str =
        "fe577e8870479194b4a9a178ac6fc434f3e3bc51aeae6a56494d7b09d5485783";
}
impl CommandAck {
    pub const KNOWN_FIELD_COUNT: usize = 3usize;
}
