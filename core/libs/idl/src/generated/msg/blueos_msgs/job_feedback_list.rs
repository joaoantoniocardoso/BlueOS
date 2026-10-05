#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JobFeedbackList {
    pub jobs: Vec<crate::msg::blueos_msgs::JobFeedback>,
}
impl CdrStruct for JobFeedbackList {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            jobs: reader.read_or_default(|reader| {
                let length = reader.read_bounded_sequence_length()?;
                let mut values = Vec::with_capacity(length as usize);
                for _index in 0..length {
                    values.push(<crate::msg::blueos_msgs::JobFeedback>::cdr_decode_fields(
                        reader,
                    )?);
                }
                Ok(values)
            })?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u32(self.jobs.len() as u32)?;
        for element in self.jobs.iter() {
            <crate::msg::blueos_msgs::JobFeedback>::cdr_encode_fields(element, writer)?;
        }
        Ok(())
    }
}
impl Message for JobFeedbackList {
    const SCHEMA: &'static str = "# blueos_msgs/msg/JobFeedbackList\n# State published on blueos/v1/<service>/jobs/<JobType>/feedback: the latest Feedback of each active Job of the type,\n# in the order the Jobs were submitted. A Job leaves it when it ends.\n\nblueos_msgs/JobFeedback[] jobs\n================================================================================\nMSG: blueos_msgs/JobFeedback\n# blueos_msgs/msg/JobFeedback\n# The latest Feedback of one active Job, in the jobs/<JobType>/feedback State (D-12, D-36), like the feedback of a ROS 2\n# action.\n\n# The UUID the client generated for the Job, as text.\nstring job_id\n# The Job type's Feedback message, CDR with its encapsulation header.\nuint8[] feedback";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/JobFeedbackList";
    const TYPE_HASH: &'static str =
        "771275894d2f14736d8021e3277558fa7e8663b2eeda3164099cd2f8ce9653e6";
}
impl JobFeedbackList {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
