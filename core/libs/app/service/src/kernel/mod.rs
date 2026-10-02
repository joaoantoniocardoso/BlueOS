//! The Kernel: the Inbox loop that applies every Command to the Domain and publishes what changed.

mod effects;
pub(crate) mod io;
mod timers;

use core::{error::Error, panic::AssertUnwindSafe};
use std::{
    panic,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use futures_util::FutureExt;
use tokio::{
    sync::{mpsc, watch},
    task::JoinSet,
};
use tracing::{error, warn};

use blueos_api::{
    CommandAck, JOB_ID_NONE, Message, cdr_encoding, command_key, event_key, info_query_key,
    query_key, settings_key, state_key, status_state_key,
};
use blueos_comms::{CommsBackend, CommsError, Query, Queryable, Sample};
use blueos_domain::{Command, Domain, Effect, Now, Outcome};
use blueos_idl::{
    Error as IdlError,
    msg::blueos_msgs::{ServiceInfo, ServiceStatus, ServiceStatusStatus, SettingsEnvelope},
};

use crate::{
    builder::{
        AnswerQuery, Decode, EventEndpoint, Refusal, Respond, ServiceBuilder, StateEndpoint,
    },
    service::ServiceError,
    settings::{SettingsDriver, settings_encoding},
};

use effects::{apply_sync_effects, io_requests};
use io::{IoExecutors, spawn_io_chain};
use timers::TimerWheel;

/// How many Commands wait in the Inbox before a sender has to wait.
const INBOX_CAPACITY: usize = 256;
/// The encoding of the reason in a Query's error reply.
const REASON_ENCODING: &str = "text/plain";

/// One applied Command's Effects in application order.
#[cfg(feature = "testing")]
type EffectBatch<D> =
    Vec<Effect<<D as Domain>::Tick, <D as Domain>::IoRequest, <D as Domain>::TimerKey>>;

/// Shared storage for recorded Effects when the harness asks not to run them.
#[cfg(feature = "testing")]
pub(crate) type EffectLogStorage<D> = Arc<std::sync::Mutex<Vec<EffectBatch<D>>>>;

/// Runs one Domain: the only writer of its Snapshot, and the owner of everything that can be stopped (the Inbox
/// and the endpoint adapters). Dropping it stops them.
pub struct Kernel<D: Domain, Context = ()> {
    service: &'static str,
    snapshot: D::Snapshot,
    inbox: mpsc::Receiver<Delivery<D>>,
    inbox_sender: Option<mpsc::Sender<Delivery<D>>>,
    states: Vec<PublishedState<D>>,
    settings: Option<SettingsEndpoint<D>>,
    events: Vec<EventEndpoint<D>>,
    backend: Arc<dyn CommsBackend>,
    clock: Arc<dyn Clock>,
    timers: TimerWheel<D>,
    context: Arc<Context>,
    io: IoExecutors<D, Context>,
    snapshot_for_queries: Arc<tokio::sync::RwLock<D::Snapshot>>,
    #[cfg(feature = "testing")]
    effect_log: Option<EffectLogStorage<D>>,
    endpoints: JoinSet<()>,
}

/// One Command in the Inbox, with an optional query to acknowledge once it is handled.
pub(crate) struct Delivery<D: Domain> {
    pub(crate) command: Command<D::Request, D::IoResult, D::Tick, D::ObservedFact>,
    pub(crate) reply: Option<Query>,
    pub(crate) persist_settings: bool,
}

/// The standard `settings` State and its persistence driver (D-11).
struct SettingsEndpoint<D: Domain> {
    key: String,
    encoding: String,
    driver: Arc<Mutex<Box<dyn SettingsDriver<D>>>>,
    latest: watch::Sender<Option<Bytes>>,
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
pub(crate) enum Rejection {
    /// The Domain rejected the Command, with its own reason.
    #[error("{0}")]
    Domain(Box<dyn Error + Send + Sync>),
    /// `handle` or a Projection panicked.
    #[error("the Command panicked, so nothing changed")]
    Panicked,
    /// The body is not the Command endpoint's Message.
    #[error("the Request does not decode: {0}")]
    InvalidBody(IdlError),
    /// The endpoint's conversion refused the Message, with its own reason.
    #[error("{0}")]
    Refused(Refusal),
}

/// Why a Query or an IO query got no answer. Its text is the reason in the error reply.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Unanswered {
    /// The body is not the endpoint's request Message.
    #[error("the Query does not decode: {0}")]
    InvalidBody(IdlError),
    /// The endpoint's conversion or IO code refused the request, with its own reason.
    #[error("{0}")]
    Refused(Refusal),
    /// The Domain answered with a Response that the endpoint does not publish.
    #[error("the Domain's Response does not belong to this Query")]
    OtherResponse,
    /// The answer panicked.
    #[error("the Query panicked")]
    Panicked,
    /// The reply does not encode.
    #[error("the reply does not encode: {0}")]
    Encode(IdlError),
}

/// Why a State, an Event or an ack did not reach the backbone.
#[derive(Debug, thiserror::Error)]
enum SendError {
    #[error("the Message does not encode: {0}")]
    Encode(#[from] IdlError),
    #[error(transparent)]
    Comms(#[from] CommsError),
}

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    /// Declares every endpoint of `builder` on `backend` and publishes the initial States, so every endpoint answers
    /// once this returns.
    ///
    /// # Errors
    ///
    /// [`ServiceError::DeclareEndpoint`] when the backbone refuses an endpoint.
    pub async fn start(
        service: &'static str,
        builder: ServiceBuilder<D, Context>,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, ServiceError> {
        #[cfg(feature = "testing")]
        {
            Self::boot(service, builder, backend, clock, None).await
        }
        #[cfg(not(feature = "testing"))]
        {
            Self::boot(service, builder, backend, clock).await
        }
    }

    /// Like [`Self::start`], optionally recording Effects without running IO or timers (harness only).
    #[cfg(feature = "testing")]
    pub async fn start_with_effect_log(
        service: &'static str,
        builder: ServiceBuilder<D, Context>,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
        effect_log: Option<EffectLogStorage<D>>,
    ) -> Result<Self, ServiceError> {
        Self::boot(service, builder, backend, clock, effect_log).await
    }

    async fn boot(
        service: &'static str,
        mut builder: ServiceBuilder<D, Context>,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
        #[cfg(feature = "testing")] effect_log: Option<EffectLogStorage<D>>,
    ) -> Result<Self, ServiceError> {
        let (inbox_sender, inbox) = mpsc::channel(INBOX_CAPACITY);
        let snapshot_for_queries = Arc::new(tokio::sync::RwLock::new(builder.snapshot.clone()));
        let mut endpoints = JoinSet::new();
        let service_info = ServiceInfo {
            name: service.to_owned(),
            version: builder.metadata.version.to_owned(),
            build: builder.metadata.build.to_owned(),
            capabilities: builder
                .metadata
                .capabilities
                .iter()
                .map(|capability| (*capability).to_owned())
                .collect(),
            endpoints: builder.manifest_endpoints.clone(),
        };
        let info_key = info_query_key(service);
        let info_encoding = cdr_encoding(ServiceInfo::SCHEMA_NAME);
        let info_payload = Bytes::from(
            service_info
                .encode()
                .map_err(|error| ServiceError::Build(Box::new(error)))?,
        );
        let info_queryable = declare(&*backend, info_key.clone()).await?;
        endpoints.spawn(serve_fixed_reply(
            info_queryable,
            info_key,
            info_payload,
            info_encoding,
        ));
        let status_key = status_state_key(service);
        let status_encoding = cdr_encoding(ServiceStatus::SCHEMA_NAME);
        let status_latest = watch::Sender::new(None);
        let status_queryable = declare(&*backend, status_key.clone()).await?;
        endpoints.spawn(serve_state(
            status_queryable,
            status_key.clone(),
            status_encoding.clone(),
            status_latest.subscribe(),
        ));
        let mut settings = None;
        if let Some(registration) = builder.settings {
            let mut driver = (registration.start)()?;
            driver.load_into(&mut builder.snapshot)?;
            let driver = Arc::new(Mutex::new(driver));
            let key = settings_key(service);
            let queryable = declare(&*backend, key.clone()).await?;
            let encoding = settings_encoding();
            let latest = watch::Sender::new(None);
            endpoints.spawn(serve_settings(
                queryable,
                key.clone(),
                encoding.clone(),
                latest.subscribe(),
            ));
            let update_key = command_key(service, "UpdateSettings");
            let update_queryable = declare(&*backend, update_key).await?;
            endpoints.spawn(serve_update_settings(
                update_queryable,
                Arc::clone(&driver),
                mpsc::Sender::clone(&inbox_sender),
            ));
            settings = Some(SettingsEndpoint {
                key,
                encoding,
                driver,
                latest,
            });
        }
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
        for (name, answer) in builder.queries {
            let queryable = declare(&*backend, query_key(service, &name)).await?;
            endpoints.spawn(serve_query::<D>(
                queryable,
                answer,
                Arc::clone(&snapshot_for_queries),
                Arc::clone(&clock),
            ));
        }
        for endpoint in builder.io_queries {
            let queryable = declare(&*backend, query_key(service, &endpoint.name)).await?;
            endpoints.spawn(serve_io_query(
                queryable,
                endpoint.respond,
                endpoint.encoding,
            ));
        }
        let kernel = Self {
            service,
            snapshot: builder.snapshot,
            inbox,
            inbox_sender: Some(inbox_sender),
            states,
            settings,
            events: builder.events,
            backend,
            clock,
            timers: TimerWheel::new(),
            context: Arc::new(builder.context),
            io: builder.io,
            snapshot_for_queries,
            #[cfg(feature = "testing")]
            effect_log,
            endpoints,
        };
        let initial_states = kernel
            .states
            .iter()
            .map(|state| (state.endpoint.project)(&kernel.snapshot))
            .collect();
        kernel.publish_states(initial_states).await;
        kernel.publish_settings().await;
        publish_standard_status(
            &kernel.backend,
            &status_key,
            &status_encoding,
            &status_latest,
        )
        .await;
        Ok(kernel)
    }

    /// Handles the Commands in the Inbox one at a time, until every endpoint has stopped.
    pub async fn run(mut self) {
        while !self.endpoints.is_empty() {
            tokio::select! {
                delivery = self.inbox.recv() => {
                    if let Some(delivery) = delivery {
                        self.dispatch(delivery).await;
                    }
                }
                tick = self.timers.next_tick(), if self.timers.waiting() => {
                    if let Some(tick) = tick {
                        self.dispatch(Delivery {
                            command: Command::Tick(tick),
                            reply: None,
                            persist_settings: false,
                        })
                        .await;
                    }
                }
                _ = self.endpoints.join_next() => {}
            }
        }
        self.inbox_sender.take();
        while let Some(delivery) = self.inbox.recv().await {
            self.dispatch(delivery).await;
        }
    }

    /// Applies one Command as a transaction: if the Domain rejects it, or `handle` or a Projection panics, the
    /// Snapshot is restored from a clone taken first and the domain events are dropped.
    async fn dispatch(&mut self, delivery: Delivery<D>) {
        let Delivery {
            command,
            reply,
            persist_settings,
        } = delivery;
        let now = self.clock.now();
        let backup = self.snapshot.clone();
        let snapshot = &mut self.snapshot;
        let states = &self.states;
        #[cfg(feature = "testing")]
        let run_effects = self.effect_log.is_none();
        #[cfg(not(feature = "testing"))]
        let run_effects = true;
        let timers = &mut self.timers;
        let io = &self.io;
        let decided = panic::catch_unwind(AssertUnwindSafe(|| {
            match D::handle(snapshot, command, now) {
                Outcome::Applied { events, effects } => {
                    apply_sync_effects(&effects, timers, io, run_effects)
                        .map_err(|error| Rejection::Domain(Box::new(error)))?;
                    let encoded_states: Vec<_> = states
                        .iter()
                        .map(|state| (state.endpoint.project)(snapshot))
                        .collect();
                    Ok((events, effects, encoded_states))
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
            Ok((events, effects, encoded_states)) => {
                {
                    let mut shared = self.snapshot_for_queries.write().await;
                    *shared = self.snapshot.clone();
                }
                #[cfg(feature = "testing")]
                if let Some(log) = &self.effect_log {
                    log.lock()
                        .expect("the effect log mutex is not poisoned")
                        .push(effects.clone());
                }
                if persist_settings && let Some(settings) = &self.settings {
                    let persist_result = settings
                        .driver
                        .lock()
                        .expect("settings driver mutex is not poisoned")
                        .persist(&self.snapshot);
                    match persist_result {
                        Ok(()) => settings
                            .driver
                            .lock()
                            .expect("settings driver mutex is not poisoned")
                            .commit_persisted(&self.snapshot),
                        Err(error) => {
                            self.snapshot = backup;
                            if let Some(query) = reply {
                                acknowledge(query, Err(Rejection::Domain(error.into()))).await;
                            }
                            return;
                        }
                    }
                }
                self.publish_states(encoded_states).await;
                self.publish_settings().await;
                if let Some(query) = reply {
                    acknowledge(query, Ok(())).await;
                }
                self.publish_events(events).await;
                if run_effects {
                    let requests = io_requests::<D>(&effects);
                    if let (Some(inbox_sender), false) = (&self.inbox_sender, requests.is_empty()) {
                        spawn_io_chain(
                            self.io.clone(),
                            Arc::clone(&self.context),
                            self.snapshot.clone(),
                            requests,
                            mpsc::Sender::clone(inbox_sender),
                        );
                    }
                }
            }
            Err(rejection) => {
                self.snapshot = backup;
                if let Some(query) = reply {
                    acknowledge(query, Err(rejection)).await;
                }
            }
        }
    }

    async fn publish_settings(&self) {
        let Some(settings) = &self.settings else {
            return;
        };
        let sent: Result<(), SendError> = async {
            let payload = Bytes::from(
                settings
                    .driver
                    .lock()
                    .expect("settings driver mutex is not poisoned")
                    .encode_state(&self.snapshot)?,
            );
            if settings.latest.borrow().as_ref() == Some(&payload) {
                return Ok(());
            }
            let sample = Sample::new(
                settings.key.as_str(),
                Bytes::clone(&payload),
                &settings.encoding,
            );
            self.backend.publish(sample).await?;
            settings.latest.send_replace(Some(payload));
            Ok(())
        }
        .await;
        warn_on_failure("State", &settings.key, sent);
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

async fn publish_standard_status(
    backend: &Arc<dyn CommsBackend>,
    key: &str,
    encoding: &str,
    latest: &watch::Sender<Option<Bytes>>,
) {
    let status = ServiceStatus {
        status: ServiceStatusStatus::Ready,
        detail: String::new(),
    };
    let sent: Result<(), SendError> = async {
        let payload = Bytes::from(status.encode()?);
        if latest.borrow().as_ref() == Some(&payload) {
            return Ok(());
        }
        let sample = Sample::new(key, Bytes::clone(&payload), encoding);
        backend.publish(sample).await?;
        latest.send_replace(Some(payload));
        Ok(())
    }
    .await;
    warn_on_failure("State", key, sent);
}

/// Answers every `info` query with the same encoded [`ServiceInfo`].
async fn serve_fixed_reply(
    mut queryable: Queryable,
    key: String,
    payload: Bytes,
    encoding: String,
) {
    while let Some(query) = queryable.recv().await {
        let sent = query.reply(Bytes::clone(&payload), encoding.as_str()).await;
        warn_on_failure("Query", &key, sent.map_err(SendError::from));
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
                    reply: Some(query),
                    persist_settings: false,
                };
                drop(inbox.send(delivery).await);
            }
            Err(rejection) => acknowledge(query, Err(rejection)).await,
        }
    }
}

/// Decodes `UpdateSettings`, validates the document, and queues a Domain Command that persists on success.
async fn serve_update_settings<D: Domain>(
    mut queryable: Queryable,
    driver: Arc<Mutex<Box<dyn SettingsDriver<D>>>>,
    inbox: mpsc::Sender<Delivery<D>>,
) {
    while let Some(query) = queryable.recv().await {
        let body = query.body().map(|body| body.payload().to_bytes());
        let bytes = body.unwrap_or_default();
        let decoded = match SettingsEnvelope::decode(&bytes) {
            Ok(envelope) => envelope,
            Err(error) => {
                acknowledge(query, Err(Rejection::InvalidBody(error))).await;
                continue;
            }
        };
        let request = driver
            .lock()
            .expect("settings driver mutex is not poisoned")
            .request_from_envelope(decoded);
        match request {
            Ok(request) => {
                let delivery = Delivery {
                    command: Command::Request(request),
                    reply: Some(query),
                    persist_settings: true,
                };
                drop(inbox.send(delivery).await);
            }
            Err(error) => acknowledge(query, Err(Rejection::Domain(error))).await,
        }
    }
}

/// Answers every get on the `settings` State with the last value the backbone accepted.
async fn serve_settings(
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

/// Answers every get on a Domain Query endpoint from the current Snapshot.
async fn serve_query<D: Domain>(
    mut queryable: Queryable,
    answer: AnswerQuery<D>,
    snapshot: Arc<tokio::sync::RwLock<D::Snapshot>>,
    clock: Arc<dyn Clock>,
) {
    while let Some(query) = queryable.recv().await {
        let body = query
            .body()
            .map(|body| body.payload().to_bytes())
            .unwrap_or_default();
        let now = clock.now();
        let answered = {
            let shared = snapshot.read().await;
            panic::catch_unwind(AssertUnwindSafe(|| answer(&shared, &body, now)))
                .unwrap_or(Err(Unanswered::Panicked))
        };
        reply(query, answered).await;
    }
}

/// Answers each IO query in turn, so one slow answer delays the next instead of running beside it.
async fn serve_io_query(mut queryable: Queryable, respond: Respond, encoding: String) {
    while let Some(query) = queryable.recv().await {
        let body = query
            .body()
            .map(|body| body.payload().to_bytes().into_owned());
        let answered = AssertUnwindSafe(respond(body.unwrap_or_default()))
            .catch_unwind()
            .await
            .unwrap_or(Err(Unanswered::Panicked));
        reply(query, answered.map(|payload| (payload, encoding.clone()))).await;
    }
}

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

/// Sends a Query's answer, or an error reply whose payload is the reason it got none.
async fn reply(query: Query, answered: Result<(Vec<u8>, String), Unanswered>) {
    let key = query.key_expression().to_owned();
    let sent = match answered {
        Ok((payload, encoding)) => query.reply(payload, encoding).await,
        Err(unanswered) => {
            let reason = unanswered.to_string().into_bytes();
            query.reply_error(reason, REASON_ENCODING).await
        }
    };
    warn_on_failure("Query", &key, sent.map_err(SendError::from));
}

/// A failed publish or reply is logged and never stops the Inbox loop.
fn warn_on_failure(kind: &'static str, key: &str, sent: Result<(), SendError>) {
    if let Err(error) = sent {
        warn!(%error, kind, key, "Failed to send");
    }
}
