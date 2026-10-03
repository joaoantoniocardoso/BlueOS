//! The Inbox: Commands waiting for the Kernel loop.

use tokio::sync::oneshot;

use blueos_api::CommandAck;
use blueos_comms::Query;
use blueos_domain::Domain;
use blueos_jobs::{JobControl, JobId, JobNature};

use crate::builder::InboxCommand;

/// Where the Kernel sends a Command's verdict.
pub(crate) enum CommandReply {
    /// A client Command endpoint query to answer on the backbone.
    Query(Query),
    /// An in-process [`CommandSender::send_awaiting_ack`] waiter.
    Ack(oneshot::Sender<CommandAck>),
}

/// One Command in the Inbox, with an optional place to deliver the verdict.
pub(crate) struct Delivery<D: Domain> {
    pub(crate) input: Input<D>,
    pub(crate) reply: Option<CommandReply>,
    pub(crate) persist_settings: bool,
}

/// What the Kernel applies: a Command for the Domain, or a client's Job submit or control (D-36).
pub(crate) enum Input<D: Domain> {
    /// A Command that is no Job: an IO result, a Tick, an Observed fact, or a Request from in-process code.
    Command(InboxCommand<D>),
    /// A client submits the Job `job_id`, whose Goal decoded into `request`.
    Submit {
        job_id: JobId,
        job_type: String,
        goal: Vec<u8>,
        nature: JobNature,
        request: D::Request,
    },
    /// A client controls the Job `job_id`.
    Control { job_id: JobId, control: JobControl },
}

impl<D: Domain> Input<D> {
    /// The Job a client named, if any.
    pub(crate) const fn job_id(&self) -> Option<JobId> {
        match self {
            Self::Command(_command) => None,
            Self::Submit { job_id, .. } | Self::Control { job_id, .. } => Some(*job_id),
        }
    }
}
