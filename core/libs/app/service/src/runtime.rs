use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blueos_api::{
    cdr_encoding, command_key, event_key, info_query_key, jobs_key, query_key, settings_key,
    status_state_key,
};
use blueos_cli::Argv;
use blueos_comms::{Payload, Session, StateHandle};
use blueos_cqrs::{App, Domain, Effect, TimerId};
use blueos_idl::Message;
use blueos_idl::msg::blueos_msgs::{CommandAck, ServiceInfo, SettingField, SettingsEnvelope};
use blueos_idl::msg::foxglove_msgs::Log as FoxgloveLog;
use blueos_jobs::JobId;
use blueos_logging::{LogRecord, attach_zenoh_publisher, log_key_for_service};
use blueos_settings::{SettingsManager, SettingsSchema, serialize_settings_document};
use bytes::Bytes;
use futures::StreamExt;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{error, instrument, warn};

use crate::error::ServiceError;
use crate::shutdown::IoInflight;

const SHUTDOWN_IO_DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

struct TimerRegistration {
    cancelled: Arc<AtomicBool>,
    task: tokio::task::JoinHandle<()>,
}

pub type CommandDecode<D> =
    Arc<dyn Fn(&[u8]) -> Result<<D as Domain>::Command, String> + Send + Sync>;
pub type QueryHandler<D> =
    Arc<dyn Fn(&[u8], &App<D>) -> Result<(Payload, String), String> + Send + Sync>;
pub type IoQueryHandler = Arc<
    dyn Fn(Payload) -> Pin<Box<dyn Future<Output = Result<(Payload, String), String>> + Send>>
        + Send
        + Sync,
>;
pub type StatePublish<D> = Arc<dyn Fn(&App<D>) -> Result<(Payload, String), String> + Send + Sync>;
pub type EventPublish<D> =
    Arc<dyn Fn(&<D as Domain>::Event) -> Result<(Payload, String), String> + Send + Sync>;
pub type EventFilter<D> = Arc<dyn Fn(&<D as Domain>::Event) -> bool + Send + Sync>;
pub type IoExecutor<D> = Arc<
    dyn Fn(
            App<D>,
            <D as Domain>::IoRequest,
        ) -> Pin<
            Box<dyn Future<Output = Result<<D as Domain>::Command, <D as Domain>::Command>> + Send>,
        > + Send
        + Sync,
>;

pub type CliCommandMapper<D> =
    Arc<dyn Fn(&[String]) -> Result<<D as Domain>::Command, String> + Send + Sync>;

pub struct CommandRegistration<D: Domain> {
    pub key: String,
    pub decode: CommandDecode<D>,
}

pub struct QueryRegistration<D: Domain> {
    pub name: String,
    pub handler: QueryHandler<D>,
}

pub struct IoQueryRegistration {
    pub name: String,
    pub handler: IoQueryHandler,
}

pub struct EventRegistration<D: Domain> {
    pub name: String,
    pub filter: EventFilter<D>,
    pub publish: EventPublish<D>,
}

pub enum SettingsSlot<D: Domain> {
    None,
    Ready(Arc<dyn SettingsRuntime<D>>),
}

pub trait SettingsRuntime<D: Domain>: Send + Sync {
    fn settings_state(&self, application: &App<D>) -> Result<(Payload, String), ServiceError>;
    fn persist_blocking(&self, application: &App<D>) -> Result<(), ServiceError>;
    fn update_settings_decode(&self, payload: &[u8]) -> Result<D::Command, String>;
}

pub fn build_settings_runtime<D, S>(
    service_name: String,
    config_folder: Option<PathBuf>,
    to_command: Arc<dyn Fn(SettingsEnvelope) -> Result<D::Command, String> + Send + Sync>,
    from_app: SettingsFromApp<D, S>,
) -> Result<Arc<dyn SettingsRuntime<D>>, blueos_settings::SettingsError>
where
    D: Domain + 'static,
    S: SettingsSchema + Send + Sync + 'static,
    D::Snapshot: Clone,
    D::JobSpec: Clone,
{
    let manager = SettingsManager::<S>::new(service_name, config_folder)?;
    Ok(Arc::new(TypedSettingsRuntime {
        manager: Arc::new(std::sync::Mutex::new(manager)),
        from_app,
        to_command,
    }))
}

type SettingsFromApp<D, S> = Arc<dyn Fn(&App<D>) -> S + Send + Sync>;

struct TypedSettingsRuntime<D: Domain, S: SettingsSchema> {
    manager: Arc<std::sync::Mutex<SettingsManager<S>>>,
    from_app: SettingsFromApp<D, S>,
    to_command: Arc<dyn Fn(SettingsEnvelope) -> Result<D::Command, String> + Send + Sync>,
}

impl<D, S> SettingsRuntime<D> for TypedSettingsRuntime<D, S>
where
    D: Domain,
    S: SettingsSchema + Send + Sync + 'static,
{
    fn settings_state(&self, application: &App<D>) -> Result<(Payload, String), ServiceError> {
        settings_envelope(&(self.from_app)(application))
    }

    fn persist_blocking(&self, application: &App<D>) -> Result<(), ServiceError> {
        let settings = (self.from_app)(application);
        let mut manager = self
            .manager
            .lock()
            .map_err(|_| ServiceError::Message("settings manager poisoned".into()))?;
        *manager.settings_mut() = settings;
        manager.save().map_err(ServiceError::from)
    }

    fn update_settings_decode(&self, payload: &[u8]) -> Result<D::Command, String> {
        let envelope = SettingsEnvelope::decode(payload).map_err(|error| error.to_string())?;
        (self.to_command)(envelope)
    }
}

enum InboxMessage<D: Domain> {
    Command {
        command: D::Command,
        reply: Option<oneshot::Sender<CommandAck>>,
    },
    IoComplete {
        command: D::Command,
    },
    Query {
        query_index: usize,
        payload: Payload,
        reply: oneshot::Sender<Result<(Payload, String), String>>,
    },
}

struct KernelState<D: Domain> {
    service_name: String,
    application: App<D>,
    timers: HashMap<TimerId, TimerRegistration>,
    io_executor: Option<IoExecutor<D>>,
    settings: Option<Arc<dyn SettingsRuntime<D>>>,
    status: Option<StatePublish<D>>,
    jobs: Option<StatePublish<D>>,
    extra_states: Vec<StatePublish<D>>,
    events: Vec<EventRegistration<D>>,
    query_handlers: Vec<QueryHandler<D>>,
    session: Session,
    status_handle: Option<StateHandle>,
    jobs_handle: Option<StateHandle>,
    settings_handle: Option<StateHandle>,
    extra_handles: Vec<StateHandle>,
    pending_events: Vec<D::Event>,
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip_all, fields(service = %service_name))]
pub async fn run<D: Domain + 'static>(
    service_name: String,
    service_info: ServiceInfo,
    application: App<D>,
    session: Session,
    cli: Argv,
    mut commands: Vec<CommandRegistration<D>>,
    queries: Vec<QueryRegistration<D>>,
    io_queries: Vec<IoQueryRegistration>,
    extra_states: Vec<(String, StatePublish<D>)>,
    events: Vec<EventRegistration<D>>,
    status: Option<StatePublish<D>>,
    jobs: Option<StatePublish<D>>,
    settings_slot: SettingsSlot<D>,
    io_executor: Option<IoExecutor<D>>,
    cli_command: Option<CliCommandMapper<D>>,
    startup_commands: Vec<D::Command>,
    shutdown_command: Option<D::Command>,
    mut shutdown_receiver: Option<watch::Receiver<bool>>,
) -> Result<(), ServiceError>
where
    D::Command: Send + Clone + 'static,
    D::Snapshot: Clone,
    D::JobSpec: Clone,
{
    let _liveliness = session
        .declare_liveliness(&blueos_api::service_liveliness_key(&service_name))
        .await?;

    let _log_guard = attach_log_publisher(&session, &service_name).await?;

    let status_handle = if status.is_some() {
        Some(
            session
                .declare_state(&status_state_key(&service_name))
                .await?,
        )
    } else {
        None
    };

    let jobs_handle = if jobs.is_some() {
        Some(session.declare_state(&jobs_key(&service_name)).await?)
    } else {
        None
    };

    let settings_runtime = match settings_slot {
        SettingsSlot::None => None,
        SettingsSlot::Ready(runtime) => Some(runtime),
    };
    let settings_handle = if settings_runtime.is_some() {
        Some(session.declare_state(&settings_key(&service_name)).await?)
    } else {
        None
    };

    if let Some(settings) = settings_runtime.clone() {
        let decode = {
            let settings = Arc::clone(&settings);
            Arc::new(move |payload: &[u8]| settings.update_settings_decode(payload))
                as CommandDecode<D>
        };
        commands.push(CommandRegistration {
            key: command_key(&service_name, "UpdateSettings"),
            decode,
        });
    }

    let mut extra_handles = Vec::new();
    let mut extra_publishers = Vec::new();
    for (name, publish) in extra_states {
        let handle = session
            .declare_state(&blueos_api::state_key(&service_name, &name))
            .await?;
        extra_handles.push(handle);
        extra_publishers.push(publish);
    }

    let info_key = info_query_key(&service_name);
    let info_queryable = session.declare_queryable(&info_key).await?;
    let service_info = Arc::new(service_info);
    tokio::spawn(async move {
        let mut stream = info_queryable;
        while let Some(query) = stream.next().await {
            let reply = match message_to_payload(service_info.as_ref()) {
                Ok((payload, encoding)) => query.reply(payload, &encoding).await,
                Err(message) => query.reply_error(&message).await,
            };
            if let Err(error) = reply {
                error!("info query reply failed: {error}");
            }
        }
    });

    let query_registrations: Vec<(String, QueryHandler<D>)> = queries
        .into_iter()
        .map(|registration| (registration.name, registration.handler))
        .collect();
    let query_handlers: Vec<QueryHandler<D>> = query_registrations
        .iter()
        .map(|(_, handler)| Arc::clone(handler))
        .collect();

    let (inbox_sender, mut inbox_receiver) = mpsc::channel::<InboxMessage<D>>(256);

    let (persist_sender, mut persist_receiver) = mpsc::channel::<App<D>>(64);
    let persist_worker = settings_runtime.clone().map(|settings| {
        tokio::spawn(async move {
            while let Some(application) = persist_receiver.recv().await {
                let settings = Arc::clone(&settings);
                let persist_result =
                    tokio::task::spawn_blocking(move || settings.persist_blocking(&application))
                        .await;
                match persist_result {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => error!("settings persist failed: {error}"),
                    Err(error) => error!("settings persist task join failed: {error}"),
                }
            }
        })
    });

    let io_inflight = IoInflight::new();
    let dispatch_context = DispatchContext {
        persist_sender: if settings_runtime.is_some() {
            Some(persist_sender)
        } else {
            None
        },
        io_inflight: io_inflight.clone(),
    };

    let mut kernel = KernelState {
        service_name: service_name.clone(),
        application,
        timers: HashMap::new(),
        io_executor,
        settings: settings_runtime,
        status,
        jobs,
        extra_states: extra_publishers,
        events,
        query_handlers,
        session: session.clone(),
        status_handle,
        jobs_handle,
        settings_handle,
        extra_handles,
        pending_events: Vec::new(),
    };

    publish_all_states(&mut kernel).await?;

    for (index, (name, _handler)) in query_registrations.iter().enumerate() {
        spawn_query_adapter(
            inbox_sender.clone(),
            session.clone(),
            query_key(&service_name, name),
            index,
        );
    }

    let io_query_shutdown = shutdown_receiver.clone();
    for registration in io_queries {
        spawn_io_query_adapter(
            session.clone(),
            query_key(&service_name, &registration.name),
            registration.handler,
            io_query_shutdown.clone(),
        );
    }

    for registration in commands {
        spawn_command_adapter(
            session.clone(),
            inbox_sender.clone(),
            registration.key,
            registration.decode,
        );
    }

    if let Some(mapper) = cli_command
        && let Ok(command) = mapper(&cli.args)
        && let Err(error) = inbox_sender
            .send(InboxMessage::Command {
                command,
                reply: None,
            })
            .await
    {
        error!("cli command inbox send failed: {error}");
    }

    for command in startup_commands {
        if let Err(error) = inbox_sender
            .send(InboxMessage::Command {
                command,
                reply: None,
            })
            .await
        {
            error!("startup command inbox send failed: {error}");
        }
    }

    let mut shutting_down = false;
    let mut shutdown_deadline: Option<tokio::time::Instant> = None;
    // Created once: a signal arriving while no listener exists would be lost, since tokio replaces the default
    // disposition on first registration.
    let shutdown_signal = wait_for_shutdown_signal(&mut shutdown_receiver);
    tokio::pin!(shutdown_signal);

    while !shutting_down || io_inflight.count() > 0 {
        if shutting_down
            && let Some(deadline) = shutdown_deadline
            && tokio::time::Instant::now() >= deadline
        {
            warn!(
                "shutdown io drain timed out after {:?}",
                SHUTDOWN_IO_DRAIN_TIMEOUT
            );
            break;
        }

        let message = if shutting_down {
            let remaining = shutdown_deadline
                .map(|deadline| deadline.saturating_duration_since(tokio::time::Instant::now()))
                .unwrap_or(SHUTDOWN_IO_DRAIN_TIMEOUT);
            match tokio::time::timeout(remaining, inbox_receiver.recv()).await {
                Ok(message) => message,
                Err(_) => break,
            }
        } else {
            tokio::select! {
                message = inbox_receiver.recv() => message,
                _ = &mut shutdown_signal => {
                    begin_shutdown(
                        &mut kernel,
                        &inbox_sender,
                        &dispatch_context,
                        shutdown_command.clone(),
                    )
                    .await?;
                    shutting_down = true;
                    shutdown_deadline =
                        Some(tokio::time::Instant::now() + SHUTDOWN_IO_DRAIN_TIMEOUT);
                    continue;
                }
            }
        };

        match message {
            None => break,
            Some(InboxMessage::Command { command, reply }) => {
                if shutting_down {
                    if let Some(sender) = reply
                        && sender
                            .send(rejected_ack("service shutting down".into()))
                            .is_err()
                    {
                        error!("shutdown reject ack dropped");
                    }
                    continue;
                }
                match dispatch_command(
                    &mut kernel,
                    command,
                    inbox_sender.clone(),
                    &dispatch_context,
                )
                .await
                {
                    Ok(CommandDispatch::Applied) => {
                        if let Some(sender) = reply
                            && sender.send(accepted_ack(&kernel.application)).is_err()
                        {
                            error!("command ack dropped");
                        }
                        publish_all_states(&mut kernel).await?;
                        publish_pending_events(&mut kernel).await?;
                    }
                    Ok(CommandDispatch::Rejected(reason)) => {
                        if let Some(sender) = reply {
                            if sender.send(rejected_ack(reason)).is_err() {
                                error!("command reject ack dropped");
                            }
                        } else {
                            warn!(
                                service = %kernel.service_name,
                                "domain rejected internal command: {reason}"
                            );
                        }
                    }
                    Err(error) => {
                        error!("command handling failed: {error}");
                        if let Some(sender) = reply
                            && sender.send(rejected_ack(error.to_string())).is_err()
                        {
                            error!("command reject ack dropped");
                        }
                    }
                }
            }
            Some(InboxMessage::IoComplete { command }) => {
                match dispatch_command(
                    &mut kernel,
                    command,
                    inbox_sender.clone(),
                    &dispatch_context,
                )
                .await
                {
                    Ok(CommandDispatch::Applied) => {
                        publish_all_states(&mut kernel).await?;
                        publish_pending_events(&mut kernel).await?;
                    }
                    Ok(CommandDispatch::Rejected(reason)) => {
                        warn!(
                            service = %kernel.service_name,
                            "domain rejected internal command: {reason}"
                        );
                    }
                    Err(error) => error!("io completion failed: {error}"),
                }
            }
            Some(InboxMessage::Query {
                query_index,
                payload,
                reply,
            }) => {
                if shutting_down {
                    if reply.send(Err("service shutting down".into())).is_err() {
                        error!("shutdown query reject dropped");
                    }
                    continue;
                }
                let payload_bytes = payload.to_vec();
                let response = kernel
                    .query_handlers
                    .get(query_index)
                    .ok_or_else(|| "unknown query".to_string())
                    .and_then(|handler| handler(&payload_bytes, &kernel.application));
                if reply.send(response).is_err() {
                    error!("query reply dropped");
                }
            }
        }

        if shutting_down && io_inflight.count() == 0 {
            break;
        }
    }

    // Closing the persist queue lets the worker flush pending settings writes before the process exits.
    drop(dispatch_context);
    if let Some(worker) = persist_worker
        && tokio::time::timeout(SHUTDOWN_IO_DRAIN_TIMEOUT, worker)
            .await
            .is_err()
    {
        warn!("settings persist drain timed out after {SHUTDOWN_IO_DRAIN_TIMEOUT:?}");
    }

    Ok(())
}

struct DispatchContext<D: Domain> {
    persist_sender: Option<mpsc::Sender<App<D>>>,
    io_inflight: IoInflight,
}

async fn wait_for_shutdown_signal(shutdown_receiver: &mut Option<watch::Receiver<bool>>) {
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    #[cfg(unix)]
    let mut sigterm =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).ok();

    // A failed signal listener (e.g. its driver's runtime is gone) must never read as a shutdown request.
    tokio::select! {
        _ = async {
            if let Err(error) = ctrl_c.as_mut().await {
                warn!("SIGINT listener failed: {error}");
                std::future::pending::<()>().await;
            }
        } => {}
        _ = async {
            if let Some(ref mut sigterm) = sigterm
                && sigterm.recv().await.is_some()
            {
                return;
            }
            std::future::pending::<()>().await;
        }, if sigterm.is_some() => {}
        _ = async {
            if let Some(receiver) = shutdown_receiver {
                while !*receiver.borrow_and_update() {
                    if receiver.changed().await.is_err() {
                        std::future::pending::<()>().await;
                    }
                }
            } else {
                std::future::pending::<()>().await;
            }
        }, if shutdown_receiver.is_some() => {}
    }
}

async fn begin_shutdown<D: Domain + 'static>(
    kernel: &mut KernelState<D>,
    inbox_sender: &mpsc::Sender<InboxMessage<D>>,
    dispatch_context: &DispatchContext<D>,
    shutdown_command: Option<D::Command>,
) -> Result<(), ServiceError>
where
    D::Command: Clone,
    D::Snapshot: Clone,
    D::JobSpec: Clone,
{
    if let Some(command) = shutdown_command {
        match dispatch_command(kernel, command, inbox_sender.clone(), dispatch_context).await? {
            CommandDispatch::Applied => {
                publish_all_states(kernel).await?;
                publish_pending_events(kernel).await?;
            }
            CommandDispatch::Rejected(reason) => {
                warn!(
                    service = %kernel.service_name,
                    "domain rejected shutdown command: {reason}"
                );
            }
        }
    }
    Ok(())
}

fn spawn_command_adapter<D: Domain + 'static>(
    session: Session,
    inbox_sender: mpsc::Sender<InboxMessage<D>>,
    key: String,
    decode: CommandDecode<D>,
) where
    D::Command: Send,
{
    tokio::spawn(async move {
        let Ok(mut queries) = session.declare_queryable(&key).await else {
            return;
        };
        while let Some(query) = queries.next().await {
            let payload = query.payload.as_slice();
            let command = match decode(&payload) {
                Ok(command) => command,
                Err(reason) => {
                    let ack = rejected_ack(reason);
                    let Ok((payload, encoding)) = command_ack_payload(&ack) else {
                        if let Err(error) = query.reply_error("ack encode failed").await {
                            error!("command reply_error failed: {error}");
                        }
                        continue;
                    };
                    if let Err(error) = query.reply(payload, &encoding).await {
                        error!("command reply failed: {error}");
                    }
                    continue;
                }
            };
            let (reply_sender, reply_receiver) = oneshot::channel();
            if inbox_sender
                .send(InboxMessage::Command {
                    command,
                    reply: Some(reply_sender),
                })
                .await
                .is_err()
            {
                if let Err(error) = query.reply_error("inbox closed").await {
                    error!("command reply_error failed: {error}");
                }
                continue;
            }
            match reply_receiver.await {
                Ok(ack) => match command_ack_payload(&ack) {
                    Ok((payload, encoding)) => {
                        if let Err(error) = query.reply(payload, &encoding).await {
                            error!("command reply failed: {error}");
                        }
                    }
                    Err(message) => {
                        if let Err(error) = query.reply_error(&message).await {
                            error!("command reply_error failed: {error}");
                        }
                    }
                },
                Err(_) => {
                    if let Err(error) = query.reply_error("inbox dropped ack").await {
                        error!("command reply_error failed: {error}");
                    }
                }
            }
        }
    });
}

fn spawn_io_query_adapter(
    session: Session,
    key: String,
    handler: IoQueryHandler,
    mut shutdown_receiver: Option<watch::Receiver<bool>>,
) {
    // ponytail: one io_query request at a time per name; upgrade with a bounded concurrency pool if needed.
    tokio::spawn(async move {
        let Ok(mut queries) = session.declare_queryable(&key).await else {
            return;
        };
        while let Some(query) = queries.next().await {
            if io_query_shutting_down(&mut shutdown_receiver) {
                if let Err(error) = query.reply_error("service shutting down").await {
                    error!("io_query reply_error failed: {error}");
                }
                continue;
            }
            let payload = query.payload.clone();
            let response = handler(payload).await;
            match response {
                Ok((payload, encoding)) => {
                    if let Err(error) = query.reply(payload, &encoding).await {
                        error!("io_query reply failed: {error}");
                    }
                }
                Err(message) => {
                    if let Err(error) = query.reply_error(&message).await {
                        error!("io_query reply_error failed: {error}");
                    }
                }
            }
        }
    });
}

fn io_query_shutting_down(shutdown_receiver: &mut Option<watch::Receiver<bool>>) -> bool {
    match shutdown_receiver {
        Some(receiver) => *receiver.borrow(),
        None => false,
    }
}

fn spawn_query_adapter<D: Domain + 'static>(
    inbox_sender: mpsc::Sender<InboxMessage<D>>,
    session: Session,
    key: String,
    query_index: usize,
) {
    tokio::spawn(async move {
        let Ok(mut queries) = session.declare_queryable(&key).await else {
            return;
        };
        while let Some(query) = queries.next().await {
            let (reply_sender, reply_receiver) = oneshot::channel();
            if inbox_sender
                .send(InboxMessage::Query {
                    query_index,
                    payload: query.payload.clone(),
                    reply: reply_sender,
                })
                .await
                .is_err()
            {
                if let Err(error) = query.reply_error("inbox closed").await {
                    error!("query reply_error failed: {error}");
                }
                continue;
            }
            match reply_receiver.await {
                Ok(Ok((payload, encoding))) => {
                    if let Err(error) = query.reply(payload, &encoding).await {
                        error!("query reply failed: {error}");
                    }
                }
                Ok(Err(message)) => {
                    if let Err(error) = query.reply_error(&message).await {
                        error!("query reply_error failed: {error}");
                    }
                }
                Err(_) => {
                    if let Err(error) = query.reply_error("inbox dropped query").await {
                        error!("query reply_error failed: {error}");
                    }
                }
            }
        }
    });
}

enum CommandDispatch {
    Applied,
    Rejected(String),
}

#[instrument(skip_all, fields(service = %kernel.service_name))]
async fn dispatch_command<D: Domain + 'static>(
    kernel: &mut KernelState<D>,
    command: D::Command,
    inbox_sender: mpsc::Sender<InboxMessage<D>>,
    dispatch_context: &DispatchContext<D>,
) -> Result<CommandDispatch, ServiceError>
where
    D::Command: Clone,
    D::Snapshot: Clone,
    D::JobSpec: Clone,
{
    let mut pending = vec![command];
    while let Some(command) = pending.pop() {
        let decision = kernel.application.handle(command);
        if let Some(reason) = decision.rejection {
            debug_assert!(decision.events.is_empty());
            debug_assert!(decision.effects.is_empty());
            return Ok(CommandDispatch::Rejected(reason));
        }
        kernel.pending_events.extend(decision.events);
        for effect in decision.effects {
            match effect {
                Effect::Io(request) => {
                    let Some(executor) = kernel.io_executor.clone() else {
                        return Err(ServiceError::Message(
                            "Effect::Io without io adapter".into(),
                        ));
                    };
                    let application = kernel.application.clone();
                    let sender = inbox_sender.clone();
                    let io_guard = dispatch_context.io_inflight.track();
                    tokio::spawn(async move {
                        let _io_guard = io_guard;
                        let command = match executor(application, request).await {
                            Ok(command) | Err(command) => command,
                        };
                        if let Err(error) = sender.send(InboxMessage::IoComplete { command }).await
                        {
                            error!("io complete inbox send failed: {error}");
                        }
                    });
                }
                Effect::Schedule {
                    after,
                    timer,
                    command,
                } => {
                    if let Some(registration) = kernel.timers.remove(&timer) {
                        registration.cancelled.store(true, Ordering::SeqCst);
                    }
                    let cancelled = Arc::new(AtomicBool::new(false));
                    let cancelled_for_task = Arc::clone(&cancelled);
                    let sender = inbox_sender.clone();
                    let task = tokio::spawn(async move {
                        tokio::time::sleep(after).await;
                        if cancelled_for_task.load(Ordering::SeqCst) {
                            return;
                        }
                        if let Err(error) = sender
                            .send(InboxMessage::Command {
                                command,
                                reply: None,
                            })
                            .await
                        {
                            error!("scheduled command inbox send failed: {error}");
                        }
                    });
                    kernel
                        .timers
                        .insert(timer, TimerRegistration { cancelled, task });
                }
                Effect::CancelSchedule(timer) => {
                    if let Some(registration) = kernel.timers.remove(&timer) {
                        registration.cancelled.store(true, Ordering::SeqCst);
                        registration.task.abort();
                    }
                }
                Effect::Persist => {
                    let Some(sender) = dispatch_context.persist_sender.as_ref() else {
                        return Err(ServiceError::Message(
                            "Effect::Persist without settings".into(),
                        ));
                    };
                    let application = kernel.application.clone();
                    if let Err(error) = sender.try_send(application) {
                        error!("persist queue full or closed: {error}");
                    }
                }
            }
        }
    }
    Ok(CommandDispatch::Applied)
}

async fn publish_all_states<D: Domain>(kernel: &mut KernelState<D>) -> Result<(), ServiceError> {
    if let (Some(publish), Some(handle)) = (&kernel.status, &kernel.status_handle) {
        let (payload, encoding) = publish(&kernel.application).map_err(ServiceError::Message)?;
        handle.publish(payload, &encoding).await?;
    }
    if let (Some(publish), Some(handle)) = (&kernel.jobs, &kernel.jobs_handle) {
        let (payload, encoding) = publish(&kernel.application).map_err(ServiceError::Message)?;
        handle.publish(payload, &encoding).await?;
    }
    if let (Some(settings), Some(handle)) = (&kernel.settings, &kernel.settings_handle) {
        let (payload, encoding) = settings.settings_state(&kernel.application)?;
        handle.publish(payload, &encoding).await?;
    }
    for (index, publish) in kernel.extra_states.iter().enumerate() {
        let (payload, encoding) = publish(&kernel.application).map_err(ServiceError::Message)?;
        kernel.extra_handles[index]
            .publish(payload, &encoding)
            .await?;
    }
    Ok(())
}

async fn publish_pending_events<D: Domain>(
    kernel: &mut KernelState<D>,
) -> Result<(), ServiceError> {
    let events = std::mem::take(&mut kernel.pending_events);
    for event in events {
        for registration in &kernel.events {
            if !(registration.filter)(&event) {
                continue;
            }
            let (payload, encoding) =
                (registration.publish)(&event).map_err(ServiceError::Message)?;
            kernel
                .session
                .publish(
                    &event_key(&kernel.service_name, &registration.name),
                    payload,
                    &encoding,
                    None,
                )
                .await?;
        }
    }
    Ok(())
}

async fn attach_log_publisher(
    session: &Session,
    service_name: &str,
) -> Result<blueos_logging::ZenohLogGuard, ServiceError> {
    let log_key = log_key_for_service(service_name);
    attach_zenoh_publisher(session.clone(), log_key, |record: &LogRecord| {
        let message = FoxgloveLog {
            timestamp: builtin_time_now(),
            level: tracing_level_to_foxglove(record.level),
            message: record.message.clone(),
            name: record.target.clone(),
            file: record.file.clone().unwrap_or_default(),
            line: record.line.unwrap_or(0),
        };
        let bytes = message.encode().unwrap_or_default();
        (
            Payload::from_bytes(Bytes::from(bytes)),
            cdr_encoding(FoxgloveLog::SCHEMA_NAME),
        )
    })
    .await
    .map_err(ServiceError::Message)
}

fn builtin_time_now() -> blueos_idl::msg::builtin_interfaces::Time {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    blueos_idl::msg::builtin_interfaces::Time {
        sec: now.as_secs() as i32,
        nanosec: now.subsec_nanos(),
    }
}

fn tracing_level_to_foxglove(level: tracing::Level) -> u8 {
    match level {
        tracing::Level::TRACE | tracing::Level::DEBUG => 1,
        tracing::Level::INFO => 2,
        tracing::Level::WARN => 3,
        tracing::Level::ERROR => 4,
    }
}

fn accepted_ack<D: Domain>(application: &App<D>) -> CommandAck {
    let job_id = latest_root_job_id(application);
    CommandAck {
        accepted: true,
        job_id: job_id.0,
        reason: String::new(),
    }
}

fn rejected_ack(reason: String) -> CommandAck {
    CommandAck {
        accepted: false,
        job_id: 0,
        reason,
    }
}

fn command_ack_payload(ack: &CommandAck) -> Result<(Payload, String), String> {
    message_to_payload(ack)
}

fn latest_root_job_id<D: Domain>(application: &App<D>) -> JobId {
    application
        .jobs
        .snapshot()
        .jobs
        .iter()
        .filter(|job| job.parent.is_none())
        .map(|job| job.job_id)
        .max_by_key(|job_id| job_id.0)
        .unwrap_or(JobId(0))
}

fn message_to_payload<M: Message>(message: &M) -> Result<(Payload, String), String> {
    let bytes = message.encode().map_err(|error| error.to_string())?;
    Ok((
        Payload::from_bytes(Bytes::from(bytes)),
        cdr_encoding(M::SCHEMA_NAME),
    ))
}

fn settings_envelope<S: SettingsSchema>(settings: &S) -> Result<(Payload, String), ServiceError> {
    let document_json = String::from_utf8(serialize_settings_document(settings)?)
        .map_err(|error| ServiceError::Message(error.to_string()))?;
    let restart_fields: HashSet<&str> = S::restart_required_fields().iter().copied().collect();
    let value: serde_json::Value =
        serde_json::to_value(settings).map_err(|error| ServiceError::Message(error.to_string()))?;
    let object = value
        .as_object()
        .ok_or_else(|| ServiceError::Message("settings document must be a JSON object".into()))?;
    let mut fields = Vec::new();
    for key in object.keys() {
        if key == "VERSION" {
            continue;
        }
        fields.push(SettingField {
            path: key.clone(),
            restart_required: restart_fields.contains(key.as_str()),
        });
    }
    let envelope = SettingsEnvelope {
        document_json,
        fields,
    };
    message_to_payload(&envelope).map_err(ServiceError::Message)
}
