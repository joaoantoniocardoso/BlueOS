//! What a Service's `build` declares: the initial Snapshot and how the Domain meets the backbone.

use core::{error::Error, future::Future, pin::Pin};
use std::sync::Arc;

use blueos_api::{Message, cdr_encoding};
use blueos_domain::{Domain, DomainQueries, Now};
use blueos_idl::Error as IdlError;

use crate::kernel::{Rejection, Unanswered};

/// Decodes a Request body into the Domain's Request.
pub(crate) type Decode<D> = Box<dyn Fn(&[u8]) -> Result<<D as Domain>::Request, Rejection> + Send>;

/// Computes and encodes a State from the Snapshot.
pub(crate) type Project<D> =
    Box<dyn Fn(&<D as Domain>::Snapshot) -> Result<Vec<u8>, IdlError> + Send + Sync>;

/// Turns a domain event into an encoded Event, or `None` when this Event endpoint does not publish it.
pub(crate) type Select<D> =
    Box<dyn Fn(&<D as Domain>::Event) -> Option<Result<Vec<u8>, IdlError>> + Send + Sync>;

/// Decodes a Query body into the question the Inbox loop asks the Domain.
pub(crate) type Ask<D> = Box<dyn Fn(&[u8]) -> Result<Answer<D>, Unanswered> + Send>;

/// Answers one decoded Query from the Snapshot with the encoded reply.
pub(crate) type Answer<D> =
    Box<dyn FnOnce(&<D as Domain>::Snapshot, Now) -> Result<Vec<u8>, Unanswered> + Send>;

/// Answers an IO query body with the encoded reply.
pub(crate) type Respond = Box<
    dyn Fn(Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Unanswered>> + Send>> + Send,
>;

/// Why an endpoint conversion or an IO query refused what a client sent. Its text is the reason the client gets.
pub type Refusal = Box<dyn Error + Send + Sync>;

/// Everything a Service declares in `build`: the initial Snapshot, then one call per endpoint. Each call converts
/// between a Message and the Domain's own types, so the Domain never sees a Message.
#[must_use]
pub struct ServiceBuilder<D: Domain> {
    pub(crate) snapshot: D::Snapshot,
    pub(crate) commands: Vec<CommandEndpoint<D>>,
    pub(crate) queries: Vec<QueryEndpoint<D>>,
    pub(crate) io_queries: Vec<IoQueryEndpoint>,
    pub(crate) states: Vec<StateEndpoint<D>>,
    pub(crate) events: Vec<EventEndpoint<D>>,
}

/// A Command endpoint: a query on `blueos/v1/<service>/command/<name>` whose body is a Request.
pub(crate) struct CommandEndpoint<D: Domain> {
    pub(crate) name: String,
    pub(crate) decode: Decode<D>,
}

/// A Query endpoint: a query on `blueos/v1/<service>/query/<name>`, answered from the Snapshot by the Inbox loop.
pub(crate) struct QueryEndpoint<D: Domain> {
    pub(crate) name: String,
    pub(crate) encoding: String,
    pub(crate) ask: Ask<D>,
}

/// An IO query endpoint: a query on `blueos/v1/<service>/query/<name>`, answered by IO code outside the Inbox.
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

impl<D: Domain> ServiceBuilder<D> {
    /// A Service whose Domain starts from `snapshot`, with no endpoints yet.
    pub fn new(snapshot: D::Snapshot) -> Self {
        Self {
            snapshot,
            commands: Vec::new(),
            queries: Vec::new(),
            io_queries: Vec::new(),
            states: Vec::new(),
            events: Vec::new(),
        }
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

impl<D: DomainQueries> ServiceBuilder<D> {
    /// Adds the Query endpoint `name`. Its body is a `Q`, which `into_query` turns into the Domain's Query; the
    /// Inbox loop answers it between two Commands, and `into_message` turns the Domain's Response into the reply.
    /// `into_message` returns `None` for a Response that does not belong to this endpoint. A body that does not
    /// decode, a refusal, a `None` or a panic is replied as an error with its reason.
    pub fn query<Q: Message + 'static, R: Message + 'static>(
        mut self,
        name: &str,
        into_query: impl Fn(Q) -> Result<D::Query, Refusal> + Send + 'static,
        into_message: impl Fn(D::Response) -> Option<R> + Send + Sync + 'static,
    ) -> Self {
        let into_message = Arc::new(into_message);
        self.queries.push(QueryEndpoint {
            name: name.to_owned(),
            encoding: cdr_encoding(R::SCHEMA_NAME),
            ask: Box::new(move |body| {
                let question = into_query(Q::decode(body).map_err(Unanswered::InvalidBody)?)
                    .map_err(Unanswered::Refused)?;
                let into_message = Arc::clone(&into_message);
                Ok(Box::new(move |snapshot, now| {
                    into_message(D::query(snapshot, question, now))
                        .ok_or(Unanswered::OtherResponse)?
                        .encode()
                        .map_err(Unanswered::Encode)
                }))
            }),
        });
        self
    }
}
