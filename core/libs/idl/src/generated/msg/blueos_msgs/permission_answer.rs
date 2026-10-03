#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PermissionAnswer {
    pub granted: bool,
}
impl CdrStruct for PermissionAnswer {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            granted: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_bool(self.granted)?;
        Ok(())
    }
}
impl Message for PermissionAnswer {
    const SCHEMA: &'static str = "# blueos_msgs/msg/PermissionAnswer\n# The body of command/AnswerPermission, whose attachment names the Job waiting for permission (D-36).\n\n# True lets the Job execute; false cancels it.\nbool granted";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/PermissionAnswer";
    const TYPE_HASH: &'static str =
        "a1300a1191872be76c4bf26b76c9205505d3f7f474d39484979dd252dff3fbb9";
}
impl PermissionAnswer {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
