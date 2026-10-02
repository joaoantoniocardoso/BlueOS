#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::string::String;
use serde::{Deserialize, Serialize};
pub mod constants_recording_file {
    pub const STATE_RECORDING: u8 = 0u8;
    pub const STATE_READY: u8 = 1u8;
    pub const STATE_NEEDS_REPAIR: u8 = 2u8;
    pub const STATE_REPAIRING: u8 = 3u8;
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingFile {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub created: crate::msg::builtin_interfaces::Time,
    pub state: u8,
    pub repair_bytes_processed: u64,
    pub repair_total_bytes: u64,
    pub repair_bytes_per_second: f64,
    pub repair_error: String,
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
                Default::default()
            } else {
                reader.read_u8()?
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
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        writer.write_string(self.name.as_str())?;
        writer.write_u64(self.size_bytes)?;
        <crate::msg::builtin_interfaces::Time>::cdr_encode_fields(&self.created, writer)?;
        writer.write_u8(self.state)?;
        writer.write_u64(self.repair_bytes_processed)?;
        writer.write_u64(self.repair_total_bytes)?;
        writer.write_f64(self.repair_bytes_per_second)?;
        writer.write_string(self.repair_error.as_str())?;
        Ok(())
    }
}
impl Message for RecordingFile {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/RecordingFile\n# One MCAP recording in the recorder folder, as listed in RecordingLibrary.\n\n# Being written by the recorder; bytes are readable but the file has no summary yet.\nuint8 STATE_RECORDING=0\n# Finished and indexed; seekable through HTTP ranges on /userdata/recorder/<path>.\nuint8 STATE_READY=1\n# Finished without a summary (power loss, crash); RepairRecording gives it back.\nuint8 STATE_NEEDS_REPAIR=2\nuint8 STATE_REPAIRING=3\n\n# Relative to the recorder folder, forward slashes. Identifies the recording in every command.\nstring path\nstring name\nuint64 size_bytes\n# From the timestamp embedded in the file name, falling back to the file time.\nbuiltin_interfaces/Time created\nuint8 state\n# Repair progress while STATE_REPAIRING; zero otherwise.\nuint64 repair_bytes_processed\nuint64 repair_total_bytes\nfloat64 repair_bytes_per_second\n# Reason the last repair failed; empty when it did not fail. Cleared by the next repair.\nstring repair_error\n================================================================================\nMSG: builtin_interfaces/Time\n# This message communicates ROS Time defined here:\n# https://design.ros2.org/articles/clock_and_time.html\n\n# The seconds component, valid over all int32 values.\nint32 sec\n\n# The nanoseconds component, valid in the range [0, 1e9).\nuint32 nanosec";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/RecordingFile";
    const TYPE_HASH: &'static str =
        "0a853b10e0a890345cc69175792a48253a669888bfc8bfe7568d5fc8014aa29a";
}
impl RecordingFile {
    pub const KNOWN_FIELD_COUNT: usize = 9usize;
}
