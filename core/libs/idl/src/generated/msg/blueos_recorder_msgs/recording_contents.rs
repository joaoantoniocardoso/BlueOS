#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingContents {
    pub path: String,
    pub duration: crate::msg::builtin_interfaces::Duration,
    pub video_topics: Vec<String>,
    pub other_topic_count: u32,
}
impl CdrStruct for RecordingContents {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            duration: if reader.is_exhausted() {
                <crate::msg::builtin_interfaces::Duration>::default()
            } else {
                <crate::msg::builtin_interfaces::Duration>::cdr_decode_fields(reader)?
            },
            video_topics: {
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
            other_topic_count: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u32()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        <crate::msg::builtin_interfaces::Duration>::cdr_encode_fields(&self.duration, writer)?;
        writer.write_u32(self.video_topics.len() as u32)?;
        for element in self.video_topics.iter() {
            writer.write_string(element.as_str())?;
        }
        writer.write_u32(self.other_topic_count)?;
        Ok(())
    }
}
impl Message for RecordingContents {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/RecordingContents\n# What one recording of RecordingLibrary holds, read from its MCAP summary.\n\n# The path of the RecordingFile it describes.\nstring path\n# Time from the first message to the last, from the Statistics record; zero without one.\nbuiltin_interfaces/Duration duration\n# Topics of the video channels (foxglove.CompressedVideo), sorted; empty when it has no video.\nstring[] video_topics\n# How many other topics it has, such as telemetry.\nuint32 other_topic_count\n================================================================================\nMSG: builtin_interfaces/Duration\n# This message communicates ROS Duration.\n\nint32 sec\nuint32 nanosec";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/RecordingContents";
    const TYPE_HASH: &'static str =
        "7a269399f3bc3e72f064d87cfe8a9105670e17c328345ea616e5cf53fdcc4de1";
}
impl RecordingContents {
    pub const KNOWN_FIELD_COUNT: usize = 4usize;
}
