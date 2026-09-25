use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;

use blueos_api::{cdr_encoding, command_key};
use blueos_cli::Argv;
use blueos_comms::{Endpoint, Payload, Session};
use blueos_cqrs::{App, Domain};
use blueos_idl::Message;
use blueos_idl::msg::blueos_msgs::{JobList, ServiceInfo, ServiceStatus, SettingsEnvelope};
use blueos_logging::init;
use blueos_settings::SettingsSchema;
use bytes::Bytes;

use crate::error::ServiceError;
use crate::runtime::{
    CliCommandMapper, CommandRegistration, EventRegistration, IoExecutor, IoQueryRegistration,
    QueryRegistration, SettingsSlot, StatePublish, build_settings_runtime, run,
};
use crate::shutdown::{ShutdownHandle, new_shutdown_channel};

/// Builder for a BlueOS service kernel (D-04, D-12).
pub struct ServiceBuilder<D: Domain> {
    service_name: String,
    application: Option<App<D>>,
    service_info: Option<ServiceInfo>,
    verbosity: u8,
    session: Option<Session>,
    cli: Argv,
    commands: Vec<CommandRegistration<D>>,
    queries: Vec<QueryRegistration<D>>,
    io_queries: Vec<IoQueryRegistration>,
    extra_states: Vec<(String, StatePublish<D>)>,
    events: Vec<EventRegistration<D>>,
    status: Option<StatePublish<D>>,
    jobs: Option<StatePublish<D>>,
    settings: SettingsSlot<D>,
    io_executor: Option<IoExecutor<D>>,
    cli_command: Option<CliCommandMapper<D>>,
    startup_commands: Vec<D::Command>,
    shutdown_command: Option<D::Command>,
    shutdown_sender: Option<tokio::sync::watch::Sender<bool>>,
    shutdown_receiver: Option<tokio::sync::watch::Receiver<bool>>,
}

impl<D: Domain + 'static> ServiceBuilder<D> {
    /// Starts a builder for `service_name` (used in key paths and liveliness).
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            application: None,
            service_info: None,
            verbosity: 0,
            session: None,
            cli: Argv::default(),
            commands: Vec::new(),
            queries: Vec::new(),
            io_queries: Vec::new(),
            extra_states: Vec::new(),
            events: Vec::new(),
            status: None,
            jobs: None,
            settings: SettingsSlot::None,
            io_executor: None,
            cli_command: None,
            startup_commands: Vec::new(),
            shutdown_command: None,
            shutdown_sender: None,
            shutdown_receiver: None,
        }
    }

    /// Domain command dispatched through the inbox once the service starts, in registration order.
    ///
    /// Use this instead of querying the service's own command keys at startup: those queryables are declared
    /// asynchronously, so a self-query can arrive before they exist and be lost.
    pub fn on_start(mut self, command: D::Command) -> Self {
        self.startup_commands.push(command);
        self
    }

    /// Domain command dispatched on `SIGINT`, `SIGTERM`, or [`ShutdownHandle::trigger`].
    pub fn on_shutdown(mut self, command: D::Command) -> Self {
        self.shutdown_command = Some(command);
        self
    }

    /// Handle for requesting graceful shutdown in tests (no real signals).
    pub fn shutdown_handle(&mut self) -> ShutdownHandle {
        if let Some(sender) = &self.shutdown_sender {
            return ShutdownHandle::new(sender.clone());
        }
        let (handle, receiver) = new_shutdown_channel();
        self.shutdown_sender = Some(handle.sender());
        self.shutdown_receiver = Some(receiver);
        handle
    }

    /// Initial domain state (including settings copied from disk when applicable).
    pub fn app(mut self, application: App<D>) -> Self {
        self.application = Some(application);
        self
    }

    /// Metadata returned on the standard `info` query (D-12).
    pub fn service_info(mut self, service_info: ServiceInfo) -> Self {
        self.service_info = Some(service_info);
        self
    }

    /// Tracing verbosity passed to [`blueos_logging::init`].
    pub fn verbosity(mut self, verbosity: u8) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Pre-opened comms session (channel backend in tests, Zenoh in production).
    pub fn session(mut self, session: Session) -> Self {
        self.session = Some(session);
        self
    }

    /// Command-line arguments; an optional mapper turns them into an initial inbox command.
    pub fn cli(mut self, argv: Argv) -> Self {
        self.cli = argv;
        self
    }

    /// Maps parsed CLI arguments to an initial inbox command when present.
    pub fn cli_command<F>(mut self, mapper: F) -> Self
    where
        F: Fn(&[String]) -> Option<D::Command> + Send + Sync + 'static,
    {
        self.cli_command = Some(Arc::new(move |arguments| {
            mapper(arguments).ok_or_else(|| "no cli command".into())
        }));
        self
    }

    /// Python-compatible settings persistence and standard `UpdateSettings` command (D-11).
    pub fn settings<S>(
        mut self,
        config_folder: Option<PathBuf>,
        to_command: impl Fn(SettingsEnvelope) -> Result<D::Command, String> + Send + Sync + 'static,
        from_app: impl Fn(&App<D>) -> S + Send + Sync + 'static,
    ) -> Result<Self, blueos_settings::SettingsError>
    where
        S: SettingsSchema + Send + Sync + 'static,
        D::Snapshot: Clone,
        D::JobSpec: Clone,
    {
        let runtime = build_settings_runtime::<D, S>(
            self.service_name.clone(),
            config_folder,
            Arc::new(to_command),
            Arc::new(from_app),
        )?;
        self.settings = SettingsSlot::Ready(runtime);
        Ok(self)
    }

    /// Registers a command queryable at `blueos/v1/<service>/command/<name>` (D-10).
    pub fn command<F>(mut self, name: &str, decode: F) -> Self
    where
        F: Fn(&[u8]) -> Result<D::Command, String> + Send + Sync + 'static,
    {
        self.commands.push(CommandRegistration {
            key: command_key(&self.service_name, name),
            decode: Arc::new(decode),
        });
        self
    }

    /// Registers a read-side queryable at `blueos/v1/<service>/query/<name>` (D-10).
    pub fn query<F>(mut self, name: &str, handler: F) -> Self
    where
        F: Fn(&[u8], &App<D>) -> Result<(Payload, String), String> + Send + Sync + 'static,
    {
        self.queries.push(QueryRegistration {
            name: name.to_string(),
            handler: Arc::new(handler),
        });
        self
    }

    /// Registers an async read at `blueos/v1/<service>/query/<name>` handled outside the inbox (D-23).
    pub fn io_query<F, Fut>(mut self, name: &str, handler: F) -> Self
    where
        F: Fn(Payload) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(Payload, String), String>> + Send + 'static,
    {
        self.io_queries.push(IoQueryRegistration {
            name: name.to_string(),
            handler: Arc::new(move |payload| Box::pin(handler(payload))),
        });
        self
    }

    /// Publishes state after each handled command (D-10 state + stream).
    pub fn state<S, Select, Encode>(mut self, name: &str, select: Select, encode: Encode) -> Self
    where
        Select: Fn(&App<D>) -> S + Send + Sync + 'static,
        Encode: Fn(S) -> Result<(Payload, String), String> + Send + Sync + 'static,
    {
        let publish: StatePublish<D> = Arc::new(move |application| {
            let selected = select(application);
            encode(selected)
        });
        self.extra_states.push((name.to_string(), publish));
        self
    }

    /// Publishes matching domain events on `blueos/v1/<service>/event/<name>`.
    pub fn event<F, Encode>(mut self, name: &str, filter: F, encode: Encode) -> Self
    where
        F: Fn(&D::Event) -> bool + Send + Sync + 'static,
        Encode: Fn(&D::Event) -> Result<(Payload, String), String> + Send + Sync + 'static,
    {
        self.events.push(EventRegistration {
            name: name.to_string(),
            filter: Arc::new(filter),
            publish: Arc::new(encode),
        });
        self
    }

    /// Standard `status` state (D-12).
    pub fn status<F>(mut self, encode: F) -> Self
    where
        F: Fn(&App<D>) -> ServiceStatus + Send + Sync + 'static,
    {
        self.status = Some(Arc::new(move |application| {
            message_to_payload(&encode(application))
        }));
        self
    }

    /// Standard `jobs` state (D-12).
    pub fn jobs<F>(mut self, encode: F) -> Self
    where
        F: Fn(&App<D>) -> JobList + Send + Sync + 'static,
    {
        self.jobs = Some(Arc::new(move |application| {
            message_to_payload(&encode(application))
        }));
        self
    }

    /// Async IO adapter for [`Effect::Io`] (D-04).
    pub fn io<F, Fut>(mut self, executor: F) -> Self
    where
        F: Fn(App<D>, D::IoRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<D::Command, D::Command>> + Send + 'static,
    {
        self.io_executor = Some(Arc::new(move |application, request| {
            Box::pin(executor(application, request))
        }));
        self
    }

    /// Opens Zenoh (unless [`Self::session`] was set) and runs until the session closes.
    pub async fn run(mut self) -> Result<(), ServiceError> {
        if let Some(session) = self.session.take() {
            return self.run_with_session(session).await;
        }
        let session = Session::open(&self.service_name, Endpoint::Local).await?;
        self.run_with_session(session).await
    }

    /// Runs the kernel on an existing session (hermetic tests).
    pub async fn run_with_session(mut self, session: Session) -> Result<(), ServiceError> {
        let service_info = self
            .service_info
            .take()
            .ok_or_else(|| ServiceError::Message("service_info is required".into()))?;
        let application = self
            .application
            .take()
            .ok_or_else(|| ServiceError::Message("app is required".into()))?;
        init(&self.service_name, self.verbosity)?;
        run(
            self.service_name,
            service_info,
            application,
            session,
            self.cli,
            self.commands,
            self.queries,
            self.io_queries,
            self.extra_states,
            self.events,
            self.status,
            self.jobs,
            self.settings,
            self.io_executor,
            self.cli_command,
            self.startup_commands,
            self.shutdown_command,
            self.shutdown_receiver,
        )
        .await
    }
}

fn message_to_payload<M: Message>(message: &M) -> Result<(Payload, String), String> {
    let bytes = message.encode().map_err(|error| error.to_string())?;
    Ok((
        Payload::from_bytes(Bytes::from(bytes)),
        cdr_encoding(M::SCHEMA_NAME),
    ))
}
