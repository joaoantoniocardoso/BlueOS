#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingPolicy {
    pub record_mavlink_only_when_armed: bool,
    pub auto_start_recording: bool,
}
impl CdrStruct for RecordingPolicy {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            record_mavlink_only_when_armed: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
            auto_start_recording: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_bool(self.record_mavlink_only_when_armed)?;
        writer.write_bool(self.auto_start_recording)?;
        Ok(())
    }
}
impl Message for RecordingPolicy {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/RecordingPolicy\n# Persisted recorder settings (D-11).\n\nbool record_mavlink_only_when_armed\nbool auto_start_recording";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/RecordingPolicy";
    const TYPE_HASH: &'static str =
        "2b3bd233617fca46ff217864f298323f615775f18d6a8496f16c1f7eec2da62a";
}
impl RecordingPolicy {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
