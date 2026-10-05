#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FillFeedback {
    pub progress: crate::msg::fixture_msgs::Progress,
}
impl CdrStruct for FillFeedback {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            progress: reader.read_or_default(|reader| {
                <crate::msg::fixture_msgs::Progress>::cdr_decode_fields(reader)
            })?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        <crate::msg::fixture_msgs::Progress>::cdr_encode_fields(&self.progress, writer)?;
        Ok(())
    }
}
impl Message for FillFeedback {
    const SCHEMA: &'static str = "Progress progress\n================================================================================\nMSG: fixture_msgs/Progress\n# fixture_msgs/msg/Progress\nuint64 done\nuint64 total";
    const SCHEMA_NAME: &'static str = "fixture_msgs/action/Fill_Feedback";
    const TYPE_HASH: &'static str =
        "773df5a1f16b44189df06571d6ca5a695dfa58daec4ffc13428f07ceab0468f8";
}
impl FillFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
