//! The Kernel: the Inbox loop that applies every Command to the Domain and publishes what changed.

use core::{error::Error, panic::AssertUnwindSafe};
use std::{panic, sync::Arc};

use bytes::Bytes;
use tokio::{
    sync::{mpsc, watch},
    task::JoinSet,
};
use tracing::{error, warn};

use blueos_api::{
    CommandAck, JOB_ID_NONE, Message, cdr_encoding, command_key, event_key, state_key,
};
use blueos_comms::{CommsBackend, CommsError, Query, Queryable, Sample};
use blueos_domain::{Command, Domain, Now, Outcome};
use blueos_idl::Error as IdlError;

use crate::{
    builder::{Decode, EventEndpoint, ServiceBuilder, StateEndpoint},
    service::ServiceError,
};

/// How many Commands wait in the Inbox before a sender has to wait.
const INBOX_CAPACITY: usize = 256;

/// Runs one Domain: the only writer of its Snapshot, and the owner of everything that can be stopped (the Inbox
/// and the endpoint adapters). Dropping it stops them.
pub struct Kernel<D: Domain> {
    service: &'static str,
    snapshot: D::Snapshot,
    inbox: mpsc::Receiver<Delivery<D>>,
    states: Vec<PublishedState<D>>,
    events: Vec<EventEndpoint<D>>,
    backend: Arc<dyn CommsBackend>,
    clock: Arc<dyn Clock>,
    #[expect(
        dead_code,
        reason = "held so that dropping the Kernel aborts its endpoint adapters"
    )]
    endpoints: JoinSet<()>,
}

/// One Command in the Inbox, with the query to acknowledge once it is handled.
struct Delivery<D: Domain> {
    command: Command<D::Request, D::IoResult, D::Tick, D::ObservedFact>,
    query: Query,
}

/// A State with its key and the last value the backbone accepted.
struct PublishedState<D: Domain> {
    key: String,
    endpoint: StateEndpoint<D>,
    /// Stored only after a publish succeeds, so a failed publish is retried on the next Command.
    latest: watch::Sender<Option<Bytes>>,
}

/// Where the Kernel reads the time, once per Command, to give the Domain its [`Now`]. A shipped Service reads the
/// system clock; the test harness injects one that follows the paused tokio clock.
pub trait Clock: Send + Sync {
    /// The current time.
    fn now(&self) -> Now;
}

/// Why the Kernel did not apply a Command. Its text is the reason in the rejected [`CommandAck`].
#[derive(Debug, thiserror::Error)]
enum Rejection {
    /// The Domain rejected the Command, with its own reason.
    #[error("{0}")]
    Domain(Box<dyn Error + Send + Sync>),
    /// `handle` or a Projection panicked.
    #[error("the Command panicked, so nothing changed")]
    Panicked,
    /// The body is not the Command endpoint's Message.
    #[error("the Request does not decode: {0}")]
    InvalidBody(IdlError),
}

/// Why a State, an Event or an ack did not reach the backbone.
#[derive(Debug, thiserror::Error)]
enum SendError {
    #[error("the Message does not encode: {0}")]
    Encode(#[from] IdlError),
    #[error(transparent)]
    Comms(#[from] CommsError),
}

impl<D: Domain> Kernel<D> {
    /// Declares every endpoint of `builder` on `backend` and publishes the initial States, so every endpoint answers
    /// once this returns.
    ///
    /// # Errors
    ///
    /// [`ServiceError::DeclareEndpoint`] when the backbone refuses an endpoint.
    pub async fn start(
        service: &'static str,
        builder: ServiceBuilder<D>,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, ServiceError> {
        let (inbox_sender, inbox) = mpsc::channel(INBOX_CAPACITY);
        let mut endpoints = JoinSet::new();
        for command in builder.commands {
            let queryable = declare(&*backend, command_key(service, &command.name)).await?;
            endpoints.spawn(serve_command(
                queryable,
                command.decode,
                mpsc::Sender::clone(&inbox_sender),
            ));
        }
        let mut states = Vec::new();
        for endpoint in builder.states {
            let key = state_key(service, &endpoint.name);
            let queryable = declare(&*backend, key.clone()).await?;
            let latest = watch::Sender::new(None);
            endpoints.spawn(serve_state(
                queryable,
                key.clone(),
                endpoint.encoding.clone(),
                latest.subscribe(),
            ));
            states.push(PublishedState {
                key,
                endpoint,
                latest,
            });
        }
        let kernel = Self {
            service,
            snapshot: builder.snapshot,
            inbox,
            states,
            events: builder.events,
            backend,
            clock,
            endpoints,
        };
        let initial_states = kernel
            .states
            .iter()
            .map(|state| (state.endpoint.project)(&kernel.snapshot))
            .collect();
        kernel.publish_states(initial_states).await;
        Ok(kernel)
    }

    /// Handles the Commands in the Inbox one at a time, until every endpoint has stopped.
    pub async fn run(mut self) {
        while let Some(delivery) = self.inbox.recv().await {
            self.dispatch(delivery).await;
        }
    }

    /// Applies one Command as a transaction: if the Domain rejects it, or `handle` or a Projection panics, the
    /// Snapshot is restored from a clone taken first and the domain events are dropped.
    async fn dispatch(&mut self, delivery: Delivery<D>) {
        let Delivery { command, query } = delivery;
        let now = self.clock.now();
        let backup = self.snapshot.clone();
        let snapshot = &mut self.snapshot;
        let states = &self.states;
        let decided = panic::catch_unwind(AssertUnwindSafe(|| {
            match D::handle(snapshot, command, now) {
                // ponytail: the Effects are dropped until the Kernel runs IO and timers.
                Outcome::Applied { events, effects: _ } => {
                    let encoded_states: Vec<_> = states
                        .iter()
                        .map(|state| (state.endpoint.project)(snapshot))
                        .collect();
                    Ok((events, encoded_states))
                }
                Outcome::Rejected { reason } => Err(Rejection::Domain(reason)),
            }
        }))
        .unwrap_or_else(|_panic| {
            error!(
                service = self.service,
                "Command panicked, so its Snapshot was restored"
            );
            Err(Rejection::Panicked)
        });
        match decided {
            Ok((events, encoded_states)) => {
                self.publish_states(encoded_states).await;
                acknowledge(query, Ok(())).await;
                self.publish_events(events).await;
            }
            Err(rejection) => {
                self.snapshot = backup;
                acknowledge(query, Err(rejection)).await;
            }
        }
    }

    async fn publish_states(&self, encoded_states: Vec<Result<Vec<u8>, IdlError>>) {
        for (state, encoded) in self.states.iter().zip(encoded_states) {
            let sent: Result<(), SendError> = async {
                let payload = Bytes::from(encoded?);
                if state.latest.borrow().as_ref() == Some(&payload) {
                    return Ok(());
                }
                let encoding = state.endpoint.encoding.as_str();
                let sample = Sample::new(state.key.as_str(), Bytes::clone(&payload), encoding);
                self.backend.publish(sample).await?;
                state.latest.send_replace(Some(payload));
                Ok(())
            }
            .await;
            warn_on_failure("State", &state.key, sent);
        }
    }

    async fn publish_events(&self, events: Vec<D::Event>) {
        for event in events {
            for endpoint in &self.events {
                if let Some(encoded) = (endpoint.select)(&event) {
                    let key = event_key(self.service, &endpoint.name);
                    let sent: Result<(), SendError> = async {
                        let sample =
                            Sample::new(key.as_str(), encoded?, endpoint.encoding.as_str());
                        self.backend.publish(sample).await?;
                        Ok(())
                    }
                    .await;
                    warn_on_failure("Event", &key, sent);
                }
            }
        }
    }
}

async fn declare(backend: &dyn CommsBackend, key: String) -> Result<Queryable, ServiceError> {
    match backend.declare_queryable(&key).await {
        Ok(queryable) => Ok(queryable),
        Err(source) => Err(ServiceError::DeclareEndpoint { key, source }),
    }
}

/// Decodes each Request outside the Inbox loop, so a body that does not decode never reaches the Domain.
async fn serve_command<D: Domain>(
    mut queryable: Queryable,
    decode: Decode<D>,
    inbox: mpsc::Sender<Delivery<D>>,
) {
    while let Some(query) = queryable.recv().await {
        let body = query.body().map(|body| body.payload().to_bytes());
        let request = decode(&body.unwrap_or_default());
        match request {
            Ok(request) => {
                let delivery = Delivery {
                    command: Command::Request(request),
                    query,
                };
                // A closed Inbox means the Kernel stopped; dropping the query tells the client.
                drop(inbox.send(delivery).await);
            }
            Err(error) => acknowledge(query, Err(Rejection::InvalidBody(error))).await,
        }
    }
}

/// Answers every get on a State with the last value the backbone accepted.
async fn serve_state(
    mut queryable: Queryable,
    key: String,
    encoding: String,
    latest: watch::Receiver<Option<Bytes>>,
) {
    while let Some(query) = queryable.recv().await {
        let current = latest.borrow().clone();
        if let Some(payload) = current {
            let sent = query.reply(payload, encoding.as_str()).await;
            warn_on_failure("State", &key, sent.map_err(SendError::from));
        }
    }
}

async fn acknowledge(query: Query, verdict: Result<(), Rejection>) {
    let ack = match verdict {
        Ok(()) => CommandAck {
            accepted: true,
            job_id: JOB_ID_NONE,
            reason: String::new(),
        },
        Err(rejection) => CommandAck {
            accepted: false,
            job_id: JOB_ID_NONE,
            reason: rejection.to_string(),
        },
    };
    let key = query.key_expression().to_owned();
    let sent: Result<(), SendError> = async {
        let encoding = cdr_encoding(CommandAck::SCHEMA_NAME);
        query.reply(ack.encode()?, encoding).await?;
        Ok(())
    }
    .await;
    warn_on_failure("CommandAck", &key, sent);
}

/// A failed publish or reply is logged and never stops the Inbox loop.
fn warn_on_failure(kind: &'static str, key: &str, sent: Result<(), SendError>) {
    if let Err(error) = sent {
        warn!(%error, kind, key, "Failed to send");
    }
}
