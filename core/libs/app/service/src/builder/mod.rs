//! What a Service's `build` declares: the initial Snapshot and how the Domain meets the backbone.

mod endpoints;
mod job_wire;
mod register;
mod types;
mod wire;

pub(crate) use job_wire::{job_list, job_status, wire_status};
pub(crate) use types::{
    AnswerQuery, CommandEndpoint, Decode, EventEndpoint, InboxCommand, JobOutput, JobsAccess,
    MessageType, Respond, StateEndpoint,
};
pub use types::{Refusal, ServiceBuilder};
