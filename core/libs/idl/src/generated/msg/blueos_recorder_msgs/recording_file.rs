#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecordingFileState {
    #[default]
    Recording,
    Ready,
    NeedsRepair,
    Repairing,
    Unknown(u8),
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingFile {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub created: crate::msg::builtin_interfaces::Time,
    pub state: RecordingFileState,
    pub repair_bytes_processed: u64,
    pub repair_total_bytes: u64,
    pub repair_bytes_per_second: f64,
    pub repair_error: String,
    pub allowed_operations: Vec<String>,
}
impl serde::Serialize for RecordingFileState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        <u8>::serialize(&self.as_raw(), serializer)
    }
}
impl<'de> serde::Deserialize<'de> for RecordingFileState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from_raw(<u8>::deserialize(deserializer)?))
    }
}
impl RecordingFileState {
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0u8 => Self::Recording,
            1u8 => Self::Ready,
            2u8 => Self::NeedsRepair,
            3u8 => Self::Repairing,
            raw => Self::Unknown(raw),
        }
    }
    pub fn as_raw(self) -> u8 {
        match self {
            Self::Recording => 0u8,
            Self::Ready => 1u8,
            Self::NeedsRepair => 2u8,
            Self::Repairing => 3u8,
            Self::Unknown(raw) => raw,
        }
    }
}
impl CdrStruct for RecordingFile {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            name: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            size_bytes: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            created: if reader.is_exhausted() {
                <crate::msg::builtin_interfaces::Time>::default()
            } else {
                <crate::msg::builtin_interfaces::Time>::cdr_decode_fields(reader)?
            },
            state: if reader.is_exhausted() {
                <RecordingFileState>::default()
            } else {
                RecordingFileState::from_raw(reader.read_u8()?)
            },
            repair_bytes_processed: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            repair_total_bytes: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u64()?
            },
            repair_bytes_per_second: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_f64()?
            },
            repair_error: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            allowed_operations: {
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
        writer.write_string(self.path.as_str())?;
        writer.write_string(self.name.as_str())?;
        writer.write_u64(self.size_bytes)?;
        <crate::msg::builtin_interfaces::Time>::cdr_encode_fields(&self.created, writer)?;
        writer.write_u8(self.state.as_raw())?;
        writer.write_u64(self.repair_bytes_processed)?;
        writer.write_u64(self.repair_total_bytes)?;
        writer.write_f64(self.repair_bytes_per_second)?;
        writer.write_string(self.repair_error.as_str())?;
        writer.write_u32(self.allowed_operations.len() as u32)?;
        for element in self.allowed_operations.iter() {
            writer.write_string(element.as_str())?;
        }
        Ok(())
    }
}
impl Message for RecordingFile {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/RecordingFile\n# One MCAP recording in the recorder folder, as listed in RecordingLibrary.\n\n# Being written by the recorder; bytes are readable but the file has no summary yet.\nuint8 STATE_RECORDING=0\n# Finished and indexed; seekable through HTTP ranges on /userdata/recorder/<path>.\nuint8 STATE_READY=1\n# Finished without a summary (power loss, crash); RepairRecording gives it back.\nuint8 STATE_NEEDS_REPAIR=2\nuint8 STATE_REPAIRING=3\n\n# Relative to the recorder folder, forward slashes. Identifies the recording in every command.\nstring path\nstring name\nuint64 size_bytes\n# From the timestamp embedded in the file name, falling back to the file time.\nbuiltin_interfaces/Time created\nuint8 state\n# Repair progress while STATE_REPAIRING; zero otherwise.\nuint64 repair_bytes_processed\nuint64 repair_total_bytes\nfloat64 repair_bytes_per_second\n# Reason the last repair failed; empty when it did not fail. Cleared by the next repair.\nstring repair_error\n# Command endpoint names the library will accept for this file (for example DeleteRecording).\nstring[] allowed_operations\n================================================================================\nMSG: builtin_interfaces/Time\n# This message communicates ROS Time defined here:\n# https://design.ros2.org/articles/clock_and_time.html\n\n# The seconds component, valid over all int32 values.\nint32 sec\n\n# The nanoseconds component, valid in the range [0, 1e9).\nuint32 nanosec";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/RecordingFile";
    const TYPE_HASH: &'static str =
        "a2aa80f504fed4d0eb8cfba5604b9e4d28bc8842815b517c5406f6a8426e64e5";
}
impl RecordingFile {
    pub const KNOWN_FIELD_COUNT: usize = 10usize;
}
