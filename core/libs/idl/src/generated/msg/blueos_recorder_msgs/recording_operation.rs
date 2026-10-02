#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::string::String;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecordingOperationOperation {
    #[default]
    Repair,
    Snapshot,
    Delete,
    Unknown(u8),
}
impl RecordingOperationOperation {
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0u8 => Self::Repair,
            1u8 => Self::Snapshot,
            2u8 => Self::Delete,
            raw => Self::Unknown(raw),
        }
    }
    pub fn as_raw(self) -> u8 {
        match self {
            Self::Repair => 0u8,
            Self::Snapshot => 1u8,
            Self::Delete => 2u8,
            Self::Unknown(raw) => raw,
        }
    }
}
impl serde::Serialize for RecordingOperationOperation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        <u8>::serialize(&self.as_raw(), serializer)
    }
}
impl<'de> serde::Deserialize<'de> for RecordingOperationOperation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from_raw(<u8>::deserialize(deserializer)?))
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingOperation {
    pub operation: RecordingOperationOperation,
    pub path: String,
    pub output_path: String,
    pub succeeded: bool,
    pub cancelled: bool,
    pub error: String,
}
impl CdrStruct for RecordingOperation {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            operation: if reader.is_exhausted() {
                <RecordingOperationOperation>::default()
            } else {
                RecordingOperationOperation::from_raw(reader.read_u8()?)
            },
            path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            output_path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            succeeded: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
            cancelled: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
            error: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(self.operation.as_raw())?;
        writer.write_string(self.path.as_str())?;
        writer.write_string(self.output_path.as_str())?;
        writer.write_bool(self.succeeded)?;
        writer.write_bool(self.cancelled)?;
        writer.write_string(self.error.as_str())?;
        Ok(())
    }
}
impl Message for RecordingOperation {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/RecordingOperation\n# Published on blueos/v1/recorder/event/operation when a repair, snapshot or delete ends.\n\nuint8 OPERATION_REPAIR=0\nuint8 OPERATION_SNAPSHOT=1\nuint8 OPERATION_DELETE=2\n\nuint8 operation\nstring path\n# The snapshot copy; empty for other operations.\nstring output_path\nbool succeeded\nbool cancelled\nstring error";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/RecordingOperation";
    const TYPE_HASH: &'static str =
        "b0574ae89a6818ffc02eb9425de955f8c208bdc5a86697a735699d3285e213b0";
}
impl RecordingOperation {
    pub const KNOWN_FIELD_COUNT: usize = 6usize;
}
