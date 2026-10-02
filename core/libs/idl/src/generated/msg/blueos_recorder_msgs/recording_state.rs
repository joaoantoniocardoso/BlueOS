#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingState {
    pub armed: bool,
    pub session_active: bool,
    pub current_file: String,
    pub session_bytes_written: u64,
    pub recording_video_topics: Vec<String>,
}
impl CdrStruct for RecordingState {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            armed: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
            session_active: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
            current_file: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            session_bytes_written: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            recording_video_topics: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(reader.read_string()?);
                    }
                    values
                }
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_bool(self.armed)?;
        writer.write_bool(self.session_active)?;
        writer.write_string(self.current_file.as_str())?;
        writer.write_u64(self.session_bytes_written)?;
        writer.write_u32(self.recording_video_topics.len() as u32)?;
        for element in self.recording_video_topics.iter() {
            writer.write_string(element.as_str())?;
        }
        Ok(())
    }
}
impl Message for RecordingState {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/RecordingState\n# Published on blueos/v1/recorder/state/recording.\n\nbool armed\nbool session_active\nstring current_file\nuint64 session_bytes_written\nstring[] recording_video_topics";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/RecordingState";
    const TYPE_HASH: &'static str =
        "7f3bcd124971448cfeea3474ebcc4c89824cae1c5a6150c30cb80e60d080ce95";
}
impl RecordingState {
    pub const KNOWN_FIELD_COUNT: usize = 5usize;
}
