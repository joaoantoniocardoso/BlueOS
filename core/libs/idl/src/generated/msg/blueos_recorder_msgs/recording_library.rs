#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingLibrary {
    pub files: Vec<crate::msg::blueos_recorder_msgs::RecordingFile>,
}
impl CdrStruct for RecordingLibrary {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            files: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(
                            <crate::msg::blueos_recorder_msgs::RecordingFile>::cdr_decode_fields(
                                reader,
                            )?,
                        );
                    }
                    values
                }
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u32(self.files.len() as u32)?;
        for element in self.files.iter() {
            <crate::msg::blueos_recorder_msgs::RecordingFile>::cdr_encode_fields(element, writer)?;
        }
        Ok(())
    }
}
impl Message for RecordingLibrary {
    const SCHEMA: &'static str = "# blueos_recorder_msgs/msg/RecordingLibrary\n# Published on blueos/v1/recorder/state/library, newest recording first.\n\nblueos_recorder_msgs/RecordingFile[] files\n================================================================================\nMSG: builtin_interfaces/Time\n# This message communicates ROS Time defined here:\n# https://design.ros2.org/articles/clock_and_time.html\n\n# The seconds component, valid over all int32 values.\nint32 sec\n\n# The nanoseconds component, valid in the range [0, 1e9).\nuint32 nanosec\n================================================================================\nMSG: blueos_recorder_msgs/RecordingFile\n# blueos_recorder_msgs/msg/RecordingFile\n# One MCAP recording in the recorder folder, as listed in RecordingLibrary.\n\n# Being written by the recorder; bytes are readable but the file has no summary yet.\nuint8 STATE_RECORDING=0\n# Finished and indexed; seekable through HTTP ranges on /userdata/recorder/<path>.\nuint8 STATE_READY=1\n# Finished without a summary (power loss, crash); RepairRecording gives it back.\nuint8 STATE_NEEDS_REPAIR=2\nuint8 STATE_REPAIRING=3\n\n# Relative to the recorder folder, forward slashes. Identifies the recording in every command.\nstring path\nstring name\nuint64 size_bytes\n# From the timestamp embedded in the file name, falling back to the file time.\nbuiltin_interfaces/Time created\nuint8 state\n# Repair progress while STATE_REPAIRING; zero otherwise.\nuint64 repair_bytes_processed\nuint64 repair_total_bytes\nfloat64 repair_bytes_per_second\n# Reason the last repair failed; empty when it did not fail. Cleared by the next repair.\nstring repair_error\n# Command endpoint names the library will accept for this file (for example DeleteRecording).\nstring[] allowed_operations";
    const SCHEMA_NAME: &'static str = "blueos_recorder_msgs/msg/RecordingLibrary";
    const TYPE_HASH: &'static str =
        "6b267c602a74d91302a51507faf46cb02331aa4dd4082d38ff7f1e9314b873d5";
}
impl RecordingLibrary {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
