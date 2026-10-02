//! The Inbox: Commands waiting for the Kernel loop.

use tokio::sync::oneshot;

use blueos_api::CommandAck;
use blueos_comms::Query;
use blueos_domain::Domain;

/// Where the Kernel sends a Command's verdict.
pub(crate) enum CommandReply {
    /// A client Command endpoint query to answer on the backbone.
    Query(Query),
    /// An in-process [`CommandSender::send_awaiting_ack`] waiter.
    Ack(oneshot::Sender<CommandAck>),
}

/// One Command in the Inbox, with an optional place to deliver the verdict.
pub(crate) struct Delivery<D: Domain> {
    pub(crate) command: blueos_domain::Command<D::Request, D::IoResult, D::Tick, D::ObservedFact>,
    pub(crate) reply: Option<CommandReply>,
    pub(crate) persist_settings: bool,
}
