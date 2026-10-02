//! What a Service's `build` declares: the initial Snapshot and how the Domain meets the backbone.

use core::{error::Error, future::Future, pin::Pin};
use std::{path::PathBuf, sync::Arc};

use tokio::sync::watch;

use blueos_api::{Message, cdr_encoding};
use blueos_domain::{Command, Domain, DomainQueries, IoError};
use blueos_idl::{
    Error as IdlError,
    msg::blueos_msgs::{EndpointInfo, SettingsEnvelope},
};
use blueos_settings::SettingsSchema;

use crate::{
    kernel::{Rejection, Unanswered, io::IoExecutors},
    settings::{SettingsRegistration, register_settings},
    shutdown::{ShutdownHandle, new_shutdown_channel},
};

/// One Command the Kernel delivers through the Inbox.
pub(crate) type InboxCommand<D> = Command<
    <D as Domain>::Request,
    <D as Domain>::IoResult,
    <D as Domain>::Tick,
    <D as Domain>::ObservedFact,
>;

/// Decodes a Request body into the Domain's Request.
pub(crate) type Decode<D> = Box<dyn Fn(&[u8]) -> Result<<D as Domain>::Request, Rejection> + Send>;

/// Computes and encodes a State from the Snapshot.
pub(crate) type Project<D> =
    Box<dyn Fn(&<D as Domain>::Snapshot) -> Result<Vec<u8>, IdlError> + Send + Sync>;

/// Turns a domain event into an encoded Event, or `None` when this Event endpoint does not publish it.
pub(crate) type Select<D> =
    Box<dyn Fn(&<D as Domain>::Event) -> Option<Result<Vec<u8>, IdlError>> + Send + Sync>;

/// Answers an IO query body with the encoded reply.
pub(crate) type Respond = Box<
    dyn Fn(Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Unanswered>> + Send>> + Send,
>;

/// Why an endpoint conversion or an IO query refused what a client sent. Its text is the reason the client gets.
pub type Refusal = Box<dyn Error + Send + Sync>;

/// Name, version and capabilities the Kernel publishes on the standard `info` query.
#[derive(Clone, Debug)]
pub(crate) struct ServiceMetadata {
    pub(crate) version: &'static str,
    pub(crate) build: &'static str,
    pub(crate) capabilities: &'static [&'static str],
}

/// Everything a Service declares in `build`: the initial Snapshot, then one call per endpoint. Each call converts
/// between a Message and the Domain's own types, so the Domain never sees a Message.
#[must_use]
pub struct ServiceBuilder<D: Domain, Context = ()> {
    pub(crate) snapshot: D::Snapshot,
    pub(crate) context: Context,
    pub(crate) metadata: ServiceMetadata,
    pub(crate) manifest_endpoints: Vec<EndpointInfo>,
    pub(crate) io: IoExecutors<D, Context>,
    pub(crate) commands: Vec<CommandEndpoint<D>>,
    pub(crate) queries: Vec<(String, AnswerQuery<D>)>,
    pub(crate) io_queries: Vec<IoQueryEndpoint>,
    pub(crate) states: Vec<StateEndpoint<D>>,
    pub(crate) events: Vec<EventEndpoint<D>>,
    pub(crate) settings: Option<SettingsRegistration<D>>,
    pub(crate) startup_commands: Vec<InboxCommand<D>>,
    pub(crate) shutdown_request: Option<D::Request>,
    pub(crate) shutdown_sender: Option<watch::Sender<bool>>,
    pub(crate) shutdown_receiver: Option<watch::Receiver<bool>>,
}

/// Answers one Query from the Snapshot and the request body.
pub(crate) type AnswerQuery<D> = Arc<
    dyn Fn(
            &<D as Domain>::Snapshot,
            &[u8],
            blueos_domain::Now,
        ) -> Result<(Vec<u8>, String), Unanswered>
        + Send
        + Sync,
>;

/// A Command endpoint: a query on `blueos/v1/<service>/command/<name>` whose body is a Request.
pub(crate) struct CommandEndpoint<D: Domain> {
    pub(crate) name: String,
    pub(crate) decode: Decode<D>,
}

/// An IO query endpoint: a query on `blueos/v1/<service>/query/<name>`, answered outside the Inbox.
pub(crate) struct IoQueryEndpoint {
    pub(crate) name: String,
    pub(crate) encoding: String,
    pub(crate) respond: Respond,
}

/// A State: published on `blueos/v1/<service>/state/<name>` when it changes, and readable there at any time.
pub(crate) struct StateEndpoint<D: Domain> {
    pub(crate) name: String,
    pub(crate) encoding: String,
    pub(crate) project: Project<D>,
}

/// An Event endpoint: published on `blueos/v1/<service>/event/<name>`, once per domain event it selects.
pub(crate) struct EventEndpoint<D: Domain> {
    pub(crate) name: String,
    pub(crate) encoding: String,
    pub(crate) select: Select<D>,
}

impl<D: Domain> ServiceBuilder<D, ()> {
    /// A Service whose Domain starts from `snapshot`, with no endpoints yet.
    pub fn new(snapshot: D::Snapshot) -> Self {
        Self {
            snapshot,
            context: (),
            metadata: ServiceMetadata {
                version: "0.0.0",
                build: "dev",
                capabilities: &[],
            },
            manifest_endpoints: Vec::new(),
            io: IoExecutors {
                r#async: None,
                blocking: None,
            },
            commands: Vec::new(),
            queries: Vec::new(),
            io_queries: Vec::new(),
            states: Vec::new(),
            events: Vec::new(),
            settings: None,
            startup_commands: Vec::new(),
            shutdown_request: None,
            shutdown_sender: None,
            shutdown_receiver: None,
        }
    }

    /// The Context IO code receives by reference, together with the Snapshot it needs.
    pub fn context<NewContext: Send + Sync + 'static>(
        self,
        context: NewContext,
    ) -> ServiceBuilder<D, NewContext> {
        ServiceBuilder {
            snapshot: self.snapshot,
            context,
            metadata: self.metadata,
            manifest_endpoints: self.manifest_endpoints,
            io: IoExecutors {
                r#async: None,
                blocking: None,
            },
            commands: self.commands,
            queries: self.queries,
            io_queries: self.io_queries,
            states: self.states,
            events: self.events,
            settings: self.settings,
            startup_commands: self.startup_commands,
            shutdown_request: self.shutdown_request,
            shutdown_sender: self.shutdown_sender,
            shutdown_receiver: self.shutdown_receiver,
        }
    }
}

impl<D: Domain + DomainQueries, Context> ServiceBuilder<D, Context> {
    /// Adds the Query endpoint `name`. Its body is an `M`, which `into_query` turns into the Domain's Query; the
    /// answer is the `R` that `into_response` makes of the Domain's Response from the current Snapshot.
    /// `into_response` returns `None` for a Response that does not belong to this endpoint. A body that does not
    /// decode, a refusal, a `None` or a panic is replied as an error with its reason.
    pub fn query<M: Message + 'static, R: Message + 'static>(
        mut self,
        name: &str,
        into_query: impl Fn(M) -> Result<D::Query, Refusal> + Send + Sync + 'static,
        into_response: impl Fn(D::Response) -> Option<R> + Send + Sync + 'static,
    ) -> Self {
        let encoding = cdr_encoding(R::SCHEMA_NAME);
        self.queries.push((
            name.to_owned(),
            Arc::new(move |snapshot, body, now| {
                let query = into_query(M::decode(body).map_err(Unanswered::InvalidBody)?)
                    .map_err(Unanswered::Refused)?;
                let response = into_response(D::query(snapshot, query, now))
                    .ok_or(Unanswered::OtherResponse)?;
                Ok((
                    response.encode().map_err(Unanswered::Encode)?,
                    encoding.clone(),
                ))
            }),
        ));
        self
    }
}

impl<D: Domain, Context> ServiceBuilder<D, Context> {
    /// The version, build label and capabilities the Kernel publishes on the standard `info` query.
    pub fn service_metadata(
        mut self,
        version: &'static str,
        build: &'static str,
        capabilities: &'static [&'static str],
    ) -> Self {
        self.metadata = ServiceMetadata {
            version,
            build,
            capabilities,
        };
        self
    }

    /// Endpoints from the Service manifest, listed in `ServiceInfo` on the `info` query. The generated `register`
    /// sets this; the Kernel adds the standard endpoints separately.
    pub fn manifest_endpoints(mut self, manifest_endpoints: Vec<EndpointInfo>) -> Self {
        self.manifest_endpoints = manifest_endpoints;
        self
    }

    /// Domain Command dispatched through the Inbox once startup finishes, in registration order.
    ///
    /// Use this instead of querying the service's own command keys at startup: those queryables are not served until
    /// after the initial States are published and the liveliness token is declared.
    pub fn on_start(mut self, request: D::Request) -> Self {
        self.startup_commands.push(Command::Request(request));
        self
    }

    /// Domain Command dispatched on `SIGINT`, `SIGTERM`, or [`ShutdownHandle::trigger`].
    pub fn on_shutdown(mut self, request: D::Request) -> Self {
        self.shutdown_request = Some(request);
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

    /// The executor for every [`Effect::Io`]. It returns an optional IO result Command, or an [`IoError`] the Kernel
    /// turns into [`Domain::io_failed`].
    pub fn io<F, Fut>(mut self, executor: F) -> Self
    where
        F: Fn(&Context, &D::Snapshot, D::IoRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<D::IoResult>, IoError>> + Send + 'static,
    {
        self.io.r#async = Some(Arc::new(move |context, snapshot, request| {
            Box::pin(executor(context, snapshot, request))
        }));
        self
    }

    /// The executor for IO the Domain marks with [`Domain::io_runs_on_blocking_thread`]. The Kernel runs it with
    /// [`tokio::task::spawn_blocking`], with the same ordering and result reporting as [`.io`](Self::io).
    pub fn blocking_io<F>(mut self, executor: F) -> Self
    where
        F: Fn(&Context, &D::Snapshot, D::IoRequest) -> Result<Option<D::IoResult>, IoError>
            + Send
            + Sync
            + 'static,
    {
        self.io.blocking = Some(Arc::new(executor));
        self
    }

    /// Registers Python-compatible settings (D-11): the Kernel loads once at startup, owns `UpdateSettings`, and
    /// persists after each successful update.
    pub fn settings<S>(
        mut self,
        service_name: &str,
        config_folder: Option<PathBuf>,
        into_snapshot: impl Fn(&mut D::Snapshot, S) + Send + Sync + 'static,
        from_snapshot: impl Fn(&D::Snapshot) -> S + Send + Sync + 'static,
        into_request: impl Fn(SettingsEnvelope) -> Result<D::Request, Box<dyn Error + Send + Sync>>
        + Send
        + Sync
        + 'static,
    ) -> Self
    where
        S: SettingsSchema + Send + Sync + 'static,
    {
        self.settings = Some(register_settings(
            service_name.to_owned(),
            config_folder,
            into_snapshot,
            from_snapshot,
            into_request,
        ));
        self
    }

    /// Adds the Command endpoint `name`. Its body is an `M`, which `into_request` turns into the Domain's Request.
    /// A body that does not decode, or that `into_request` refuses, is rejected before it reaches the Inbox, with
    /// the refusal as the reason.
    pub fn command<M: Message + 'static>(
        mut self,
        name: &str,
        into_request: impl Fn(M) -> Result<D::Request, Refusal> + Send + 'static,
    ) -> Self {
        self.commands.push(CommandEndpoint {
            name: name.to_owned(),
            decode: Box::new(move |body| {
                into_request(M::decode(body).map_err(Rejection::InvalidBody)?)
                    .map_err(Rejection::Refused)
            }),
        });
        self
    }

    /// Adds the State `name`, computed from the Snapshot by `projection` after every applied Command and published
    /// only when its encoded value changes. `projection` must be pure: it runs inside the Command's transaction, so
    /// a panic in it restores the Snapshot and rejects the Command.
    pub fn state<M: Message + 'static>(
        mut self,
        name: &str,
        projection: impl Fn(&D::Snapshot) -> M + Send + Sync + 'static,
    ) -> Self {
        self.states.push(StateEndpoint {
            name: name.to_owned(),
            encoding: cdr_encoding(M::SCHEMA_NAME),
            project: Box::new(move |snapshot| projection(snapshot).encode()),
        });
        self
    }

    /// Adds the Event endpoint `name`, which publishes the `M` that `select` returns for a domain event, after the
    /// Command that produced it is acknowledged. `select` returns `None` for the domain events this endpoint does
    /// not publish.
    pub fn event<M: Message + 'static>(
        mut self,
        name: &str,
        select: impl Fn(&D::Event) -> Option<M> + Send + Sync + 'static,
    ) -> Self {
        self.events.push(EventEndpoint {
            name: name.to_owned(),
            encoding: cdr_encoding(M::SCHEMA_NAME),
            select: Box::new(move |event| select(event).map(|message| message.encode())),
        });
        self
    }

    /// Adds the IO query endpoint `name`, answered by `respond` outside the Inbox, one query at a time: for reads
    /// that need IO but no Domain state. A body that does not decode, a refusal or a panic in `respond` is replied
    /// as an error with its reason.
    pub fn io_query<Q: Message + 'static, R: Message + 'static>(
        mut self,
        name: &str,
        respond: impl Fn(Q) -> Pin<Box<dyn Future<Output = Result<R, Refusal>> + Send>>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        let respond = Arc::new(respond);
        self.io_queries.push(IoQueryEndpoint {
            name: name.to_owned(),
            encoding: cdr_encoding(R::SCHEMA_NAME),
            respond: Box::new(move |body| {
                let respond = Arc::clone(&respond);
                Box::pin(async move {
                    let request = Q::decode(&body).map_err(Unanswered::InvalidBody)?;
                    let response = respond(request).await.map_err(Unanswered::Refused)?;
                    response.encode().map_err(Unanswered::Encode)
                })
            }),
        });
        self
    }
}
