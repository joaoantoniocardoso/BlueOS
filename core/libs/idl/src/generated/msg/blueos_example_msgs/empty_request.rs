#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EmptyRequest {
    pub padding: u8,
}
impl CdrStruct for EmptyRequest {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            padding: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u8()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(self.padding)?;
        Ok(())
    }
}
impl Message for EmptyRequest {
    const SCHEMA: &'static str = "# blueos_example_msgs/msg/EmptyRequest\n# Command payload with no semantics (StartSelfTest, CancelSelfTest). Padding keeps CDR codegen happy.\n\nuint8 padding";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/msg/EmptyRequest";
    const TYPE_HASH: &'static str =
        "029ec6c17b5709cf24813cffef88f91fad6ed7021af1530d5fd599fee0cd69e6";
}
impl EmptyRequest {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
