#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JobFeedback {
    pub job_id: String,
    pub feedback: Vec<u8>,
}
impl CdrStruct for JobFeedback {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            job_id: reader.read_or_default(|reader| reader.read_string())?,
            feedback: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                Ok(reader.read_bytes(length as usize)?.to_vec())
            })?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.job_id.as_str())?;
        writer.write_u32(self.feedback.len() as u32)?;
        writer.write_bytes(&self.feedback)?;
        Ok(())
    }
}
impl Message for JobFeedback {
    const SCHEMA: &'static str = "# blueos_msgs/msg/JobFeedback\n# The latest Feedback of one active Job, in the jobs/<JobType>/feedback State (D-12, D-36), like the feedback of a ROS 2\n# action.\n\n# The UUID the client generated for the Job, as text.\nstring job_id\n# The Job type's Feedback message, CDR with its encapsulation header.\nuint8[] feedback";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/JobFeedback";
    const TYPE_HASH: &'static str =
        "b9853de23ae21377d2d6f1e305de830efe54c0dfc01f11a0f621a40c6281c04b";
}
impl JobFeedback {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
