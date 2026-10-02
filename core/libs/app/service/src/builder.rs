//! What a Service's `build` declares: the initial Snapshot and how the Domain meets the backbone.

use core::future::Future;
use std::sync::Arc;

use blueos_api::{Message, cdr_encoding};
use blueos_domain::{Domain, DomainQueries, IoError};
use blueos_idl::Error as IdlError;

use crate::kernel::io::IoExecutors;

/// Decodes a Request body into the Domain's Request.
pub(crate) type Decode<D> = Box<dyn Fn(&[u8]) -> Result<<D as Domain>::Request, IdlError> + Send>;

/// Computes and encodes a State from the Snapshot.
pub(crate) type Project<D> =
    Box<dyn Fn(&<D as Domain>::Snapshot) -> Result<Vec<u8>, IdlError> + Send + Sync>;

/// Turns a domain event into an encoded Event, or `None` when this Event endpoint does not publish it.
pub(crate) type Select<D> =
    Box<dyn Fn(&<D as Domain>::Event) -> Option<Result<Vec<u8>, IdlError>> + Send + Sync>;

/// Everything a Service declares in `build`: the initial Snapshot, then one call per endpoint. Each call converts
/// between a Message and the Domain's own types, so the Domain never sees a Message.
#[must_use]
pub struct ServiceBuilder<D: Domain, Context = ()> {
    pub(crate) snapshot: D::Snapshot,
    pub(crate) context: Context,
    pub(crate) io: IoExecutors<D, Context>,
    pub(crate) commands: Vec<CommandEndpoint<D>>,
    pub(crate) queries: Vec<(String, AnswerQuery<D>)>,
    pub(crate) states: Vec<StateEndpoint<D>>,
    pub(crate) events: Vec<EventEndpoint<D>>,
}

/// Answers one Query from the Snapshot and the request body.
pub(crate) type AnswerQuery<D> = Arc<
    dyn Fn(
            &<D as Domain>::Snapshot,
            &[u8],
            blueos_domain::Now,
        ) -> Result<(Vec<u8>, String), IdlError>
        + Send
        + Sync,
>;

/// A Command endpoint: a query on `blueos/v1/<service>/command/<name>` whose body is a Request.
pub(crate) struct CommandEndpoint<D: Domain> {
    pub(crate) name: String,
    pub(crate) decode: Decode<D>,
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
            io: IoExecutors {
                r#async: None,
                blocking: None,
            },
            commands: Vec::new(),
            queries: Vec::new(),
            states: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl<D: Domain> ServiceBuilder<D, ()> {
    /// The Context IO code receives by reference, together with the Snapshot it needs.
    pub fn context<NewContext: Send + Sync + 'static>(
        self,
        context: NewContext,
    ) -> ServiceBuilder<D, NewContext> {
        ServiceBuilder {
            snapshot: self.snapshot,
            context,
            io: IoExecutors {
                r#async: None,
                blocking: None,
            },
            commands: self.commands,
            queries: self.queries,
            states: self.states,
            events: self.events,
        }
    }
}

impl<D: Domain + DomainQueries, Context> ServiceBuilder<D, Context> {
    /// Adds the Query endpoint `name`. Its body is an `M`; the answer is an `R` built from the current Snapshot.
    pub fn query<M: Message + 'static, R: Message + 'static>(
        mut self,
        name: &str,
        into_query: impl Fn(M) -> D::Query + Send + Sync + 'static,
        into_response: impl Fn(D::Response) -> R + Send + Sync + 'static,
    ) -> Self {
        let encoding = cdr_encoding(R::SCHEMA_NAME);
        self.queries.push((
            name.to_owned(),
            Arc::new(move |snapshot, body, now| {
                let query = M::decode(body).map(&into_query)?;
                let response = D::query(snapshot, query, now);
                Ok((into_response(response).encode()?, encoding.clone()))
            }),
        ));
        self
    }
}

impl<D: Domain, Context> ServiceBuilder<D, Context> {
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

    /// Adds the Command endpoint `name`. Its body is an `M`, which `into_request` turns into the Domain's Request.
    /// A body that does not decode is rejected before it reaches the Inbox.
    pub fn command<M: Message + 'static>(
        mut self,
        name: &str,
        into_request: impl Fn(M) -> D::Request + Send + 'static,
    ) -> Self {
        self.commands.push(CommandEndpoint {
            name: name.to_owned(),
            decode: Box::new(move |body| M::decode(body).map(&into_request)),
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
}
