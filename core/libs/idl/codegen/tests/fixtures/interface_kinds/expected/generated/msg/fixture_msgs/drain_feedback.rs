#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DrainFeedback {
    pub progress: crate::msg::fixture_msgs::Progress,
}
impl CdrStruct for DrainFeedback {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            progress: if reader.is_exhausted() {
                <crate::msg::fixture_msgs::Progress>::default()
            } else {
                <crate::msg::fixture_msgs::Progress>::cdr_decode_fields(reader)?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        <crate::msg::fixture_msgs::Progress>::cdr_encode_fields(&self.progress, writer)?;
        Ok(())
    }
}
impl Message for DrainFeedback {
    const SCHEMA: &'static str = "Progress progress\n================================================================================\nMSG: fixture_msgs/Progress\n# fixture_msgs/msg/Progress\nuint64 done\nuint64 total";
    const SCHEMA_NAME: &'static str = "fixture_msgs/action/Drain_Feedback";
    const TYPE_HASH: &'static str =
        "8a67aa568cb654c96bfe036eece5334c4a7340c459d5441fbf4e9fb42888a8ce";
}
impl DrainFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
