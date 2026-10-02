//! Puts Commands into the service's own Inbox from in-process code (D-27).

use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};

use blueos_api::{CommandAck, JOB_ID_NONE};
use blueos_domain::Domain;
use blueos_jobs::JobId;

use crate::{
    builder::InboxCommand,
    inbox::{CommandReply, Delivery},
    kernel::Rejection,
};

/// The service's connection to the backbone (D-27).
pub type Session = Arc<dyn blueos_comms::CommsBackend>;

/// Why a Command could not be queued.
#[derive(Debug, thiserror::Error)]
pub enum SendError {
    /// [`CommandSender::try_send`] found no free Inbox slot.
    #[error("the Inbox is full")]
    InboxFull,
    /// The Kernel stopped and closed the Inbox.
    #[error("the Inbox is closed")]
    Closed,
    /// [`CommandSender::send_awaiting_ack`] did not receive a verdict.
    #[error("the Inbox dropped the Command's verdict")]
    AckDropped,
}

/// Sends Commands into this service's Inbox. Clone it for every Task or adapter that needs one.
pub struct CommandSender<D: Domain> {
    inbox: mpsc::Sender<Delivery<D>>,
}

impl<D: Domain> Clone for CommandSender<D> {
    fn clone(&self) -> Self {
        Self {
            inbox: self.inbox.clone(),
        }
    }
}

impl<D: Domain> CommandSender<D> {
    pub(crate) fn new(inbox: mpsc::Sender<Delivery<D>>) -> Self {
        Self { inbox }
    }

    /// Queues `command`. Waits when the Inbox is full.
    pub async fn send(&self, command: InboxCommand<D>) -> Result<(), SendError> {
        let delivery = Delivery {
            command,
            reply: None,
            persist_settings: false,
        };
        self.inbox
            .send(delivery)
            .await
            .map_err(|_error| SendError::Closed)
    }

    /// Queues `command` without waiting. Fails fast when the Inbox is full.
    pub fn try_send(&self, command: InboxCommand<D>) -> Result<(), SendError> {
        let delivery = Delivery {
            command,
            reply: None,
            persist_settings: false,
        };
        self.inbox.try_send(delivery).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_delivery) => SendError::InboxFull,
            mpsc::error::TrySendError::Closed(_delivery) => SendError::Closed,
        })
    }

    /// Queues `command` and waits for the Domain's verdict, with the same ack shape as an external Command.
    pub async fn send_awaiting_ack(
        &self,
        command: InboxCommand<D>,
    ) -> Result<CommandAck, SendError> {
        let (verdict_sender, verdict_receiver) = oneshot::channel();
        let delivery = Delivery {
            command,
            reply: Some(CommandReply::Ack(verdict_sender)),
            persist_settings: false,
        };
        self.inbox
            .send(delivery)
            .await
            .map_err(|_error| SendError::Closed)?;
        verdict_receiver
            .await
            .map_err(|_error| SendError::AckDropped)
    }
}

pub(crate) fn command_ack(verdict: Result<Option<JobId>, Rejection>) -> CommandAck {
    match verdict {
        Ok(started) => CommandAck {
            accepted: true,
            job_id: started.map_or(JOB_ID_NONE, JobId::get),
            reason: String::new(),
        },
        Err(rejection) => CommandAck {
            accepted: false,
            job_id: JOB_ID_NONE,
            reason: rejection.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use blueos_domain::{Command, Domain, Now, Outcome};

    use super::*;

    enum StubRequest {
        One,
    }

    struct StubDomain;

    impl Domain for StubDomain {
        type Snapshot = ();
        type Request = StubRequest;
        type Event = Infallible;
        type IoResult = Infallible;
        type Tick = Infallible;
        type ObservedFact = Infallible;
        type IoRequest = Infallible;
        type TimerKey = Infallible;

        fn handle(
            _snapshot: &mut Self::Snapshot,
            command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
            _now: Now,
        ) -> blueos_domain::Decision<Self> {
            match command {
                Command::Request(StubRequest::One) => Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                },
                Command::IoResult(io_result) => match io_result {},
                Command::Tick(tick) => match tick {},
                Command::ObservedFact(observed_fact) => match observed_fact {},
            }
        }

        fn io_failed(
            request: Self::IoRequest,
            _error: blueos_domain::IoError,
        ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
            match request {}
        }
    }

    #[tokio::test]
    async fn try_send_fails_fast_when_the_inbox_is_full() {
        let (inbox, _receiver) = mpsc::channel(1);
        let sender = CommandSender::<StubDomain>::new(inbox);
        sender
            .send(Command::Request(StubRequest::One))
            .await
            .expect("the first Command is queued");
        let full = sender.try_send(Command::Request(StubRequest::One));
        assert!(matches!(full, Err(SendError::InboxFull)));
    }
}
