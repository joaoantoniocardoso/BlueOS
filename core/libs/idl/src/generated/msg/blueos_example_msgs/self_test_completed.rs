#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SelfTestCompleted {
    pub passed: bool,
    pub detail: String,
    #[serde(with = "serde_arrays")]
    pub checks: [bool; 3usize],
}
impl CdrStruct for SelfTestCompleted {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            passed: reader.read_or_default(|reader| reader.read_bool())?,
            detail: reader.read_or_default(|reader| reader.read_string())?,
            checks: reader.read_or_default(|reader| {
                let mut values = [Default::default(); 3usize];
                for value in &mut values {
                    *value = reader.read_bool()?;
                }
                Ok(values)
            })?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_bool(self.passed)?;
        writer.write_string(self.detail.as_str())?;
        for element in self.checks.iter() {
            writer.write_bool(*element)?;
        }
        Ok(())
    }
}
impl Message for SelfTestCompleted {
    const SCHEMA: &'static str = "# blueos_example_msgs/msg/SelfTestCompleted\n# Event on blueos/v1/example/event/SelfTestCompleted.\n\nbool passed\nstring detail\n# One result per check, in the order the self-test runs them: motor, seal, level sensor.\nbool[3] checks";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/msg/SelfTestCompleted";
    const TYPE_HASH: &'static str =
        "47df47cbdf07c139db9a34ec2e6f2467b1abc29268fcb337b8a514babec44b37";
}
impl SelfTestCompleted {
    pub const KNOWN_FIELD_COUNT: usize = 3usize;
}
