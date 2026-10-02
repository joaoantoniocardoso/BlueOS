#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::string::String;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SelfTestCompleted {
    pub passed: bool,
    pub detail: String,
}
impl CdrStruct for SelfTestCompleted {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            passed: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
            detail: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_bool(self.passed)?;
        writer.write_string(self.detail.as_str())?;
        Ok(())
    }
}
impl Message for SelfTestCompleted {
    const SCHEMA: &'static str = "# blueos_example_msgs/msg/SelfTestCompleted\n# Event on blueos/v1/example/event/SelfTestCompleted.\n\nbool passed\nstring detail";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/msg/SelfTestCompleted";
    const TYPE_HASH: &'static str =
        "3dfad74638ed7382bbf3dba7fbdb9fe3c026bf01c0e3f1c0cefb9b39d6023710";
}
impl SelfTestCompleted {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
