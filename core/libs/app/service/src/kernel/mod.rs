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
use tracing::warn;

use blueos_api::{
    CommandAck, Message, cdr_encoding, command_key, event_key, info_query_key, jobs_key, query_key,
    service_liveliness_key, settings_key, state_key, status_state_key,
};
use blueos_comms::{CommsBackend, CommsError, Query, Queryable, Sample};
use blueos_domain::{Command, Domain, Outcome};
use blueos_idl::{
    Error as IdlError,
    msg::blueos_msgs::{
        EndpointInfo, JobList, ServiceInfo, ServiceStatus, ServiceStatusStatus, SettingsEnvelope,
    },
};
use blueos_jobs::JobId;

use crate::{
    builder::{
        AnswerQuery, Decode, EventEndpoint, LatestRoot, Refusal, Respond, ServiceBuilder,
        StateEndpoint,
    },
    clock::Clock,
    command_sender::{CommandSender, Session, command_ack},
    durable_state::{DurablePersister, DurableStateHandle},
    inbox::{CommandReply, Delivery},
    inbox_recovery::{self, log_caught_panic},
    logging::LogPublisherRuntime,
    projection::ProjectionRegistry,
    run_outcome::RunOutcome,
    service::ServiceError,
    settings::{SettingsDriver, settings_encoding},
    shutdown::{IoInflight, SHUTDOWN_IO_DRAIN_TIMEOUT, wait_for_shutdown_signal},
    sync::lock_unpoisoned,
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
type EffectBatch<D> = Vec<
    blueos_domain::Effect<<D as Domain>::Tick, <D as Domain>::IoRequest, <D as Domain>::TimerKey>,
>;

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
    durable: Option<DurableStateHandle<D>>,
    events: Vec<EventEndpoint<D>>,
    /// Set only for a Domain with Jobs.
    latest_root: Option<LatestRoot<D>>,
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
    log_publisher: Option<LogPublisherRuntime>,
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
    /// once this returns. The Kernel owns `context` from here on and hands it to IO code and Tasks.
    ///
    /// # Errors
    ///
    /// [`ServiceError::DeclareEndpoint`] when the backbone refuses an endpoint.
    pub async fn start(
        service: &'static str,
        builder: ServiceBuilder<D, Context>,
        context: Context,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, ServiceError> {
        #[cfg(feature = "testing")]
        {
            Self::boot(service, builder, context, backend, clock, None).await
        }
        #[cfg(not(feature = "testing"))]
        {
            Self::boot(service, builder, context, backend, clock).await
        }
    }

    /// Like [`Self::start`], optionally recording Effects without running IO or timers (harness only).
    #[cfg(feature = "testing")]
    pub async fn start_with_effect_log(
        service: &'static str,
        builder: ServiceBuilder<D, Context>,
        context: Context,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
        effect_log: Option<EffectLogStorage<D>>,
    ) -> Result<Self, ServiceError> {
        Self::boot(service, builder, context, backend, clock, effect_log).await
    }

    /// Runs the service log publisher flush as the last shutdown step (D-04, D-13).
    pub fn attach_log_publisher(&mut self, runtime: LogPublisherRuntime) {
        self.log_publisher = Some(runtime);
    }

    /// Handle for waiting on debounced durable writes in tests.
    #[cfg(feature = "testing")]
    pub fn durable_write_flush(&self) -> Option<crate::durable_state::DurableWriteFlush> {
        self.durable
            .as_ref()
            .map(|durable| durable.persister.flush_handle())
    }

    async fn boot(
        service: &'static str,
        mut builder: ServiceBuilder<D, Context>,
        context: Context,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
        #[cfg(feature = "testing")] effect_log: Option<EffectLogStorage<D>>,
    ) -> Result<Self, ServiceError> {
        let mut startup_commands = builder.startup_commands;
        let shutdown_request = tokio::sync::Mutex::new(builder.shutdown_request);
        let durable_registration = builder.durable.take();
        if let Some(registration) = &durable_registration {
            registration
                .store
                .ensure_directory()
                .map_err(ServiceError::Settings)?;
            if (registration.restore_from_disk)(&mut builder.snapshot, clock.as_ref()) {
                startup_commands.push(Command::Tick(registration.restored_tick.clone()));
            }
        }
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
            endpoints: builder
                .manifest_endpoints
                .iter()
                .cloned()
                .chain(builder.jobs.as_ref().map(|_jobs| EndpointInfo {
                    kind: "state".to_owned(),
                    name: "jobs".to_owned(),
                    key: jobs_key(service),
                    request_schema: String::new(),
                    response_schema: JobList::SCHEMA_NAME.to_owned(),
                }))
                .collect(),
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
            service,
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
        let latest_root = builder.jobs.as_ref().map(|jobs| jobs.latest_root);
        let state_endpoints = builder
            .states
            .into_iter()
            .map(|endpoint| (state_key(service, &endpoint.name), endpoint))
            .chain(builder.jobs.map(|jobs| (jobs_key(service), jobs.state)));
        let mut states = Vec::new();
        let mut pending_states = Vec::new();
        for (key, endpoint) in state_endpoints {
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
        let durable = durable_registration.map(|registration| DurableStateHandle {
            persister: DurablePersister::spawn(Arc::clone(&clock), registration.store),
            serialize: registration.serialize,
            changed: registration.changed,
        });
        let session: Session = Arc::clone(&backend);
        let mut kernel = Self {
            service,
            snapshot: builder.snapshot,
            inbox,
            inbox_sender: Some(inbox_sender),
            states,
            settings,
            durable,
            events: builder.events,
            latest_root,
            backend,
            clock,
            timers: TimerWheel::new(),
            context: Arc::new(context),
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
            log_publisher: None,
        };
        kernel.projections.refresh(&kernel.snapshot);
        for command in startup_commands {
            if kernel
                .dispatch(Delivery {
                    command,
                    reply: None,
                    persist_settings: false,
                })
                .await
                .is_some()
            {
                return Err(ServiceError::Build(
                    "the Kernel stopped during startup after repeated Inbox panics".into(),
                ));
            }
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
        let mut repeated_inbox_panics = None;
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
                    if let Some(outcome) = self.dispatch(delivery).await {
                        repeated_inbox_panics = Some(outcome);
                        break;
                    }
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
                            if let Some(outcome) = self.dispatch(delivery).await {
                                repeated_inbox_panics = Some(outcome);
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                if repeated_inbox_panics.is_some() {
                    break;
                }
                continue;
            }

            if repeated_inbox_panics.is_some() {
                break;
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
                        if let Some(outcome) = self.dispatch(delivery).await {
                            repeated_inbox_panics = Some(outcome);
                        }
                    } else if self.endpoints.is_empty() {
                        break;
                    }
                }
                tick = self.timers.next_tick(), if self.timers.waiting() => {
                    if let Some(tick) = tick
                        && let Some(outcome) = self.dispatch(Delivery {
                            command: Command::Tick(tick),
                            reply: None,
                            persist_settings: false,
                        })
                        .await
                    {
                        repeated_inbox_panics = Some(outcome);
                    }
                }
                _ = self.endpoints.join_next(), if !self.endpoints.is_empty() => {}
            }
            if repeated_inbox_panics.is_some() {
                break;
            }
        }

        self.inbox_sender.take();
        self.endpoints.abort_all();
        while let Ok(Some(delivery)) = tokio::time::timeout(Duration::ZERO, self.inbox.recv()).await
        {
            if let Some(outcome) = self.dispatch(delivery).await {
                repeated_inbox_panics = Some(outcome);
                break;
            }
        }
        if let Some(durable) = &mut self.durable {
            durable.persister.flush_and_shutdown().await;
        }
        if let Some(runtime) = self.log_publisher.take() {
            runtime.shutdown_and_wait().await;
        }
        repeated_inbox_panics.unwrap_or(RunOutcome::Stopped)
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
    ///
    /// Returns [`RunOutcome::RepeatedInboxPanics`] when the rolling panic budget is exhausted (D-29).
    async fn dispatch(&mut self, delivery: Delivery<D>) -> Option<RunOutcome> {
        let unwound = AssertUnwindSafe(self.dispatch_delivery(delivery))
            .catch_unwind()
            .await;
        match unwound {
            Ok(maybe_stop) => maybe_stop,
            Err(panic) => {
                log_caught_panic(self.service, None, panic);
                if self
                    .tasks
                    .record_inbox_loop_panic(self.clock.now().monotonic)
                    .await
                {
                    Some(RunOutcome::RepeatedInboxPanics)
                } else {
                    None
                }
            }
        }
    }

    async fn dispatch_delivery(&mut self, delivery: Delivery<D>) -> Option<RunOutcome> {
        let Delivery {
            command,
            reply,
            persist_settings,
        } = delivery;
        if self.shutting_down && reply.is_some() {
            complete_command_reply(reply, Err(Rejection::ShuttingDown)).await;
            return None;
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
        .unwrap_or_else(|panic| {
            log_caught_panic(self.service, Some(inbox_recovery::INBOX_LOOP_NAME), panic);
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
                    lock_unpoisoned(log).push(effects.clone());
                }
                if persist_settings && let Some(settings) = &self.settings {
                    let persist_result = lock_unpoisoned(&settings.driver).persist(&self.snapshot);
                    match persist_result {
                        Ok(()) => {
                            lock_unpoisoned(&settings.driver).commit_persisted(&self.snapshot)
                        }
                        Err(error) => {
                            self.snapshot = backup;
                            complete_command_reply(reply, Err(Rejection::Domain(error.into())))
                                .await;
                            return None;
                        }
                    }
                }
                if let Some(durable) = &self.durable
                    && (durable.changed)(&backup, &self.snapshot)
                {
                    durable
                        .persister
                        .queue_document((durable.serialize)(&self.snapshot));
                }
                self.publish_states(encoded_states).await;
                self.publish_settings().await;
                self.projections.refresh(&self.snapshot);
                let started = self.latest_root.and_then(|latest_root| {
                    let latest = latest_root(&self.snapshot);
                    if latest == latest_root(&backup) {
                        None
                    } else {
                        latest
                    }
                });
                complete_command_reply(reply, Ok(started)).await;
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
                self.tasks.mark_inbox_loop_healthy().await;
                None
            }
            Err(rejection) => {
                self.snapshot = backup;
                let stop = if matches!(rejection, Rejection::Panicked) {
                    self.tasks
                        .record_inbox_loop_panic(self.clock.now().monotonic)
                        .await
                } else {
                    false
                };
                complete_command_reply(reply, Err(rejection)).await;
                if stop {
                    Some(RunOutcome::RepeatedInboxPanics)
                } else {
                    None
                }
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
            let payload =
                Bytes::from(lock_unpoisoned(&settings.driver).encode_state(&self.snapshot)?);
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
        let request = lock_unpoisoned(&driver).request_from_envelope(decoded);
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

async fn complete_command_reply(
    reply: Option<CommandReply>,
    verdict: Result<Option<JobId>, Rejection>,
) {
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
