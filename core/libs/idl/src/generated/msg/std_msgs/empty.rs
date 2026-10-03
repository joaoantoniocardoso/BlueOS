#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Empty {}
impl CdrStruct for Empty {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        if !reader.is_exhausted() {
            reader.read_u8()?;
        }
        Ok(Self {})
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(0)?;
        Ok(())
    }
}
impl Message for Empty {
    const SCHEMA: &'static str = "# A message with no fields: the body of a Job control such as CancelJob, whose attachment names the Job.";
    const SCHEMA_NAME: &'static str = "std_msgs/msg/Empty";
    const TYPE_HASH: &'static str =
        "aa564811ffb8a892afa96edd2494eea1ead213ea72e382b4b0635eb9f181e141";
}
impl Empty {
    pub const KNOWN_FIELD_COUNT: usize = 0usize;
}
