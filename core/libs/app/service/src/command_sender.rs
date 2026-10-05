//! Puts Commands into the service's own Inbox from in-process code (D-27).

use core::hash::{BuildHasher as _, Hasher as _};
use std::{hash::RandomState, sync::Arc};

use tokio::sync::{mpsc, oneshot};

use blueos_api::CommandAck;
use blueos_domain::Domain;
use blueos_idl::msg::blueos_msgs::CommandAckStatus;
use blueos_jobs::{Job, JobId};

use crate::{
    builder::{InboxCommand, wire_status},
    inbox::{CommandReply, Delivery, Input},
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
            input: Input::Command(command),
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
            input: Input::Command(command),
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
            input: Input::Command(command),
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

/// A new random Job id, as a client generates one for each Goal it submits to another Service: a UUID v4, like the
/// one the frontend's `newJobId` makes.
// ponytail: the random bits come from std's `RandomState` keys, not a cryptographic generator: ids only need to be
// unique.
#[must_use]
pub fn new_job_id() -> JobId {
    const VERSION_MASK: u128 = 0xf << 76;
    const VERSION_4: u128 = 0x4 << 76;
    const VARIANT_MASK: u128 = 0x3 << 62;
    const VARIANT_RFC_4122: u128 = 0x2 << 62;
    let random = || RandomState::new().build_hasher().finish();
    let bits = u128::from(random()) << 64 | u128::from(random());
    JobId::from_u128(bits & !VERSION_MASK & !VARIANT_MASK | VERSION_4 | VARIANT_RFC_4122)
}

/// The ack of a Command that named `job_id`, with the status of `job` after it was applied, or with no Job for a
/// Command that is no Job. An accepted ack carries the Job's reason, so a Job that ended at once says why.
pub(crate) fn command_ack(
    job_id: Option<JobId>,
    job: Option<&Job>,
    verdict: Result<(), Rejection>,
) -> CommandAck {
    CommandAck {
        accepted: verdict.is_ok(),
        job_id: job_id.map(|job_id| job_id.to_string()).unwrap_or_default(),
        status: job.map_or(CommandAckStatus::StatusUnknown, |job| {
            CommandAckStatus::from_raw(wire_status(job.status).as_raw())
        }),
        reason: match verdict {
            Ok(()) => job.map(|job| job.reason.clone()).unwrap_or_default(),
            Err(rejection) => rejection.to_string(),
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

    #[test]
    fn every_new_job_id_is_another() {
        assert_ne!(new_job_id(), new_job_id());
    }

    #[test]
    fn a_new_job_id_is_a_uuid_v4_like_the_one_a_browser_makes() {
        for _ in 0..64 {
            let job_id = new_job_id().to_string();

            assert_eq!(&job_id[14..15], "4", "{job_id}");
            assert!("89ab".contains(&job_id[19..20]), "{job_id}");
        }
    }
}
