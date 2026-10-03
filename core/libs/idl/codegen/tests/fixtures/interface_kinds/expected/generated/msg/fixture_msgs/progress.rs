#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    pub done: u64,
    pub total: u64,
}
impl CdrStruct for Progress {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            done: reader.read_or_default(|reader| reader.read_u64())?,
            total: reader.read_or_default(|reader| reader.read_u64())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u64(self.done)?;
        writer.write_u64(self.total)?;
        Ok(())
    }
}
impl Message for Progress {
    const SCHEMA: &'static str = "# fixture_msgs/msg/Progress\nuint64 done\nuint64 total";
    const SCHEMA_NAME: &'static str = "fixture_msgs/msg/Progress";
    const TYPE_HASH: &'static str =
        "d22542fd5354b4467927a5bc91d83c8e34457a36fe6a1809fea02c54894a8d4a";
}
impl Progress {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
