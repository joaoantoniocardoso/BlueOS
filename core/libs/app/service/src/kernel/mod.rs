//! The Kernel: the Inbox loop that applies every Command to the Domain and publishes what changed.

mod effects;
pub(crate) mod io;
mod timers;

use core::{error::Error, panic::AssertUnwindSafe, time::Duration};
use core::{
    future::pending,
    sync::atomic::{AtomicBool, Ordering},
};
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
    CommandAck, Message, cdr_encoding, command_key, event_key, info_query_key, query_key,
    service_liveliness_key, settings_key, state_key, status_state_key,
};
use blueos_comms::{CommsBackend, CommsError, Query, Queryable, Sample};
use blueos_domain::{Command, Domain, Effect, Outcome};
use blueos_idl::{
    Error as IdlError,
    msg::blueos_msgs::{ServiceInfo, ServiceStatus, ServiceStatusStatus, SettingsEnvelope},
};

use crate::{
    builder::{
        AnswerQuery, Decode, EventEndpoint, Refusal, Respond, ServiceBuilder, StateEndpoint,
    },
    clock::Clock,
    command_sender::{CommandSender, Session, command_ack},
    inbox::{CommandReply, Delivery},
    projection::ProjectionRegistry,
    run_outcome::RunOutcome,
    service::ServiceError,
    settings::{SettingsDriver, settings_encoding},
    shutdown::{IoInflight, SHUTDOWN_IO_DRAIN_TIMEOUT, wait_for_shutdown_signal},
    tasks::{TaskSupervisor, hold_liveliness_until_cancelled},
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
    shutdown_request: tokio::sync::Mutex<Option<D::Request>>,
    shutdown_receiver: Option<watch::Receiver<bool>>,
    io_inflight: IoInflight,
    shutting_down: bool,
    tasks: TaskSupervisor,
    projections: ProjectionRegistry<D>,
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

/// Why the Kernel did not apply a Command. Its text is the reason in the rejected [`CommandAck`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum Rejection {
    /// The Domain rejected the Command, with its own reason.
    #[error("{0}")]
    Domain(Box<dyn Error + Send + Sync>),
    /// `handle` or a Projection panicked.
    #[error("the Command panicked, so nothing changed")]
    Panicked,
    /// A client Command arrived after shutdown started.
    #[error("the service is shutting down")]
    ShuttingDown,
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
        let startup_commands = builder.startup_commands;
        let shutdown_request = tokio::sync::Mutex::new(builder.shutdown_request);
        let shutdown_receiver = builder.shutdown_receiver;
        let (inbox_sender, inbox) = mpsc::channel(INBOX_CAPACITY);
        let snapshot_for_queries = Arc::new(tokio::sync::RwLock::new(builder.snapshot.clone()));
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
        let status_key = status_state_key(service);
        let status_encoding = cdr_encoding(ServiceStatus::SCHEMA_NAME);
        let status_latest = watch::Sender::new(None);
        let status_queryable = declare(&*backend, status_key.clone()).await?;
        let task_supervisor = TaskSupervisor::new(
            Arc::clone(&backend),
            status_key.clone(),
            status_encoding.clone(),
            status_latest.clone(),
        );
        let task_specs = builder.tasks;
        let mut pending_settings_serve = None;
        let mut settings = None;
        if let Some(registration) = builder.settings {
            let mut driver = (registration.start)()?;
            driver.load_into(&mut builder.snapshot)?;
            let driver = Arc::new(Mutex::new(driver));
            let key = settings_key(service);
            let queryable = declare(&*backend, key.clone()).await?;
            let encoding = settings_encoding();
            let latest = watch::Sender::new(None);
            let update_key = command_key(service, "UpdateSettings");
            let update_queryable = declare(&*backend, update_key).await?;
            pending_settings_serve = Some((
                queryable,
                update_queryable,
                key.clone(),
                encoding.clone(),
                latest.subscribe(),
            ));
            settings = Some(SettingsEndpoint {
                key,
                encoding,
                driver,
                latest,
            });
        }
        let mut pending_commands = Vec::new();
        for command in builder.commands {
            let queryable = declare(&*backend, command_key(service, &command.name)).await?;
            pending_commands.push((queryable, command.decode));
        }
        let mut states = Vec::new();
        let mut pending_states = Vec::new();
        for endpoint in builder.states {
            let key = state_key(service, &endpoint.name);
            let queryable = declare(&*backend, key.clone()).await?;
            let latest = watch::Sender::new(None);
            pending_states.push((
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
        let mut pending_queries = Vec::new();
        for (name, answer) in builder.queries {
            let queryable = declare(&*backend, query_key(service, &name)).await?;
            pending_queries.push((queryable, answer));
        }
        let mut pending_io_queries = Vec::new();
        for endpoint in builder.io_queries {
            let queryable = declare(&*backend, query_key(service, &endpoint.name)).await?;
            pending_io_queries.push((queryable, endpoint.respond, endpoint.encoding));
        }
        let projections = ProjectionRegistry::new(builder.projections);
        let session: Session = Arc::clone(&backend);
        let mut kernel = Self {
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
            endpoints: JoinSet::new(),
            shutdown_request,
            shutdown_receiver,
            io_inflight: IoInflight::new(),
            shutting_down: false,
            tasks: task_supervisor,
            projections,
        };
        kernel.projections.refresh(&kernel.snapshot);
        for command in startup_commands {
            kernel
                .dispatch(Delivery {
                    command,
                    reply: None,
                    persist_settings: false,
                })
                .await;
        }
        let initial_states = kernel
            .states
            .iter()
            .map(|state| (state.endpoint.project)(&kernel.snapshot))
            .collect();
        kernel.publish_states(initial_states).await;
        kernel.publish_settings().await;
        kernel.projections.refresh(&kernel.snapshot);
        publish_standard_status(
            &kernel.backend,
            &status_key,
            &status_encoding,
            &status_latest,
        )
        .await;
        let command_sender = CommandSender::new(mpsc::Sender::clone(
            kernel
                .inbox_sender
                .as_ref()
                .expect("the inbox sender exists during startup"),
        ));
        kernel.tasks.start(
            task_specs,
            session,
            command_sender,
            Arc::clone(&kernel.context),
            Arc::clone(&kernel.clock),
        );
        kernel.endpoints.spawn(serve_fixed_reply(
            info_queryable,
            info_key,
            info_payload,
            info_encoding,
        ));
        kernel.endpoints.spawn(serve_state(
            status_queryable,
            status_key,
            status_encoding,
            status_latest.subscribe(),
        ));
        if let Some((queryable, update_queryable, key, encoding, latest)) = pending_settings_serve {
            let driver = Arc::clone(
                &kernel
                    .settings
                    .as_ref()
                    .expect("settings exist when their queryables were declared")
                    .driver,
            );
            kernel
                .endpoints
                .spawn(serve_settings(queryable, key, encoding, latest));
            kernel.endpoints.spawn(serve_update_settings(
                update_queryable,
                driver,
                mpsc::Sender::clone(
                    kernel
                        .inbox_sender
                        .as_ref()
                        .expect("the inbox sender exists during startup"),
                ),
            ));
        }
        for (queryable, decode) in pending_commands {
            kernel.endpoints.spawn(serve_command(
                queryable,
                decode,
                mpsc::Sender::clone(
                    kernel
                        .inbox_sender
                        .as_ref()
                        .expect("the inbox sender exists during startup"),
                ),
            ));
        }
        for (queryable, key, encoding, latest) in pending_states {
            kernel
                .endpoints
                .spawn(serve_state(queryable, key, encoding, latest));
        }
        for (queryable, answer) in pending_queries {
            kernel.endpoints.spawn(serve_query::<D>(
                queryable,
                answer,
                Arc::clone(&kernel.snapshot_for_queries),
                Arc::clone(&kernel.clock),
            ));
        }
        for (queryable, respond, encoding) in pending_io_queries {
            kernel
                .endpoints
                .spawn(serve_io_query(queryable, respond, encoding));
        }
        let liveliness_key = service_liveliness_key(service);
        let liveliness = kernel
            .backend
            .declare_liveliness(&liveliness_key)
            .await
            .map_err(|source| ServiceError::DeclareEndpoint {
                key: liveliness_key,
                source,
            })?;
        let liveliness_shutdown = kernel.tasks.shutdown_token();
        let task_spawner = kernel.tasks.spawner();
        task_spawner.spawn(async move {
            hold_liveliness_until_cancelled(liveliness, liveliness_shutdown).await;
        });
        Ok(kernel)
    }

    /// Handles the Commands in the Inbox one at a time, until shutdown finishes or every endpoint has stopped.
    pub async fn run(mut self) -> RunOutcome {
        let mut shutdown_monotonic_deadline = None;
        let mut shutdown_receiver = None;
        core::mem::swap(&mut self.shutdown_receiver, &mut shutdown_receiver);
        let stop_requested = Arc::new(AtomicBool::new(false));
        let stop_flag_for_signals = Arc::clone(&stop_requested);
        let mut signal_shutdown_receiver = shutdown_receiver.clone();
        self.tasks.spawner().spawn(async move {
            wait_for_shutdown_signal(&mut signal_shutdown_receiver).await;
            stop_flag_for_signals.store(true, Ordering::SeqCst);
        });

        loop {
            if let Some(receiver) = &mut shutdown_receiver
                && *receiver.borrow_and_update()
            {
                stop_requested.store(true, Ordering::SeqCst);
            }
            if stop_requested.load(Ordering::SeqCst) && !self.shutting_down {
                self.begin_shutdown().await;
                shutdown_monotonic_deadline =
                    Some(self.clock.now().monotonic + SHUTDOWN_IO_DRAIN_TIMEOUT);
            }

            if self.shutting_down {
                let remaining = shutdown_monotonic_deadline
                    .map(|deadline| deadline.saturating_sub(self.clock.now().monotonic))
                    .unwrap_or(SHUTDOWN_IO_DRAIN_TIMEOUT);
                while let Ok(Some(delivery)) =
                    tokio::time::timeout(Duration::ZERO, self.inbox.recv()).await
                {
                    self.dispatch(delivery).await;
                }
                if self.io_inflight.count() == 0 {
                    let task_budget = shutdown_monotonic_deadline
                        .map(|deadline| deadline.saturating_sub(self.clock.now().monotonic))
                        .unwrap_or(SHUTDOWN_IO_DRAIN_TIMEOUT);
                    self.tasks.join_with_budget(task_budget, &self.clock).await;
                    break;
                }
                if remaining == Duration::ZERO {
                    warn!(
                        timeout = ?SHUTDOWN_IO_DRAIN_TIMEOUT,
                        "shutdown io drain timed out"
                    );
                    self.tasks
                        .join_with_budget(Duration::ZERO, &self.clock)
                        .await;
                    break;
                }
                tokio::select! {
                    biased;
                    () = tokio::time::sleep(remaining) => {
                        warn!(
                            timeout = ?SHUTDOWN_IO_DRAIN_TIMEOUT,
                            "shutdown io drain timed out"
                        );
                        self.tasks.join_with_budget(Duration::ZERO, &self.clock).await;
                        break;
                    }
                    delivery = self.inbox.recv() => {
                        if let Some(delivery) = delivery {
                            self.dispatch(delivery).await;
                        } else {
                            break;
                        }
                    }
                }
                continue;
            }

            if self.endpoints.is_empty() {
                break;
            }

            tokio::select! {
                biased;
                _ = async {
                    let Some(receiver) = &mut shutdown_receiver else {
                        pending::<()>().await;
                        return;
                    };
                    while !*receiver.borrow_and_update() {
                        if receiver.changed().await.is_err() {
                            pending::<()>().await;
                        }
                    }
                }, if shutdown_receiver.is_some() => {
                    stop_requested.store(true, Ordering::SeqCst);
                }
                delivery = self.inbox.recv() => {
                    if let Some(delivery) = delivery {
                        self.dispatch(delivery).await;
                    } else if self.endpoints.is_empty() {
                        break;
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
                _ = self.endpoints.join_next(), if !self.endpoints.is_empty() => {}
            }
        }

        self.inbox_sender.take();
        self.endpoints.abort_all();
        while let Ok(Some(delivery)) = tokio::time::timeout(Duration::ZERO, self.inbox.recv()).await
        {
            self.dispatch(delivery).await;
        }
        RunOutcome::Stopped
    }

    async fn begin_shutdown(&mut self) {
        if self.shutting_down {
            return;
        }
        self.shutting_down = true;
        self.tasks.cancel();
        if let Some(request) = self.shutdown_request.lock().await.take() {
            let delivery = Delivery {
                command: Command::Request(request),
                reply: None,
                persist_settings: false,
            };
            if let Some(sender) = &self.inbox_sender {
                drop(sender.send(delivery).await);
            }
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
        if self.shutting_down && reply.is_some() {
            complete_command_reply(reply, Err(Rejection::ShuttingDown)).await;
            return;
        }
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
                            complete_command_reply(reply, Err(Rejection::Domain(error.into())))
                                .await;
                            return;
                        }
                    }
                }
                self.publish_states(encoded_states).await;
                self.publish_settings().await;
                self.projections.refresh(&self.snapshot);
                complete_command_reply(reply, Ok(())).await;
                self.publish_events(events).await;
                if run_effects {
                    let requests = io_requests::<D>(&effects);
                    if let (Some(inbox_sender), false) = (&self.inbox_sender, requests.is_empty()) {
                        spawn_io_chain(
                            self.tasks.spawner(),
                            self.io.clone(),
                            Arc::clone(&self.context),
                            self.snapshot.clone(),
                            requests,
                            mpsc::Sender::clone(inbox_sender),
                            self.io_inflight.clone(),
                        );
                    }
                }
            }
            Err(rejection) => {
                self.snapshot = backup;
                complete_command_reply(reply, Err(rejection)).await;
            }
        }
    }

    /// Hands out a [`CommandSender`] while the Inbox is still open.
    pub fn command_sender(&self) -> Option<CommandSender<D>> {
        self.inbox_sender
            .as_ref()
            .map(|sender| CommandSender::new(mpsc::Sender::clone(sender)))
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
                    reply: Some(CommandReply::Query(query)),
                    persist_settings: false,
                };
                drop(inbox.send(delivery).await);
            }
            Err(rejection) => {
                complete_command_reply(Some(CommandReply::Query(query)), Err(rejection)).await;
            }
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
                complete_command_reply(
                    Some(CommandReply::Query(query)),
                    Err(Rejection::InvalidBody(error)),
                )
                .await;
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
                    reply: Some(CommandReply::Query(query)),
                    persist_settings: true,
                };
                drop(inbox.send(delivery).await);
            }
            Err(error) => {
                complete_command_reply(
                    Some(CommandReply::Query(query)),
                    Err(Rejection::Domain(error)),
                )
                .await;
            }
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

async fn complete_command_reply(reply: Option<CommandReply>, verdict: Result<(), Rejection>) {
    let Some(reply) = reply else {
        return;
    };
    let ack = command_ack(verdict);
    match reply {
        CommandReply::Ack(sender) => {
            drop(sender.send(ack));
        }
        CommandReply::Query(query) => {
            let key = query.key_expression().to_owned();
            let sent: Result<(), SendError> = async {
                let encoding = cdr_encoding(CommandAck::SCHEMA_NAME);
                query.reply(ack.encode()?, encoding).await?;
                Ok(())
            }
            .await;
            warn_on_failure("CommandAck", &key, sent);
        }
    }
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
