//! How a Service talks on the BlueOS backbone: publish, subscribe, answer queries and query others.
//!
//! The Kernel holds one `Arc<dyn CommsBackend>`: the Zenoh backend in a shipped binary, the in-process `channel`
//! backend in tests (its feature belongs in `[dev-dependencies]` only). Payloads are never framed or copied on the
//! way through; metadata travels in the attachment.

#[cfg(feature = "channel")]
pub mod channel;

use core::{any::Any, fmt::Debug, time::Duration};
use std::{borrow::Cow, sync::Arc, time::SystemTime};

use bytes::Bytes;
use futures_util::{Stream, StreamExt, future::BoxFuture, stream::BoxStream};

/// What a get receives from one queryable: a sample on the queryable's declared key, or an error it replied.
pub type Reply = Result<Sample, ReplyError>;

type Responder = Box<dyn FnOnce(Reply) -> BoxFuture<'static, Result<(), CommsError>> + Send>;

/// Publishing, subscribing and querying on the backbone. A Service shares one as `Arc<dyn CommsBackend>`; in a
/// shipped binary it is the Service's Session.
///
/// Key expressions follow Zenoh: `*` matches one chunk, `**` any number of chunks and `$*` part of a chunk. They
/// must be in canonical form (`a/**`, not `a/**/**`).
pub trait CommsBackend: Send + Sync {
    /// Sends `sample` to every subscriber whose key expression intersects its key.
    fn publish(&self, sample: Sample) -> BoxFuture<'_, Result<(), CommsError>>;

    /// Receives every sample published on a key that intersects `key_expression`, until the [`Subscriber`] is
    /// dropped.
    fn subscribe<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<Subscriber, CommsError>>;

    /// Receives every get whose key expression intersects `key`, until the [`Queryable`] is dropped. Replies
    /// always carry `key`, so a wildcard get tells them apart.
    fn declare_queryable<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<Queryable, CommsError>>;

    /// Sends a query to every queryable whose key intersects `key_expression` and returns their replies in
    /// arrival order, once each of them has replied or dropped its query, or when `timeout` expires.
    ///
    /// A get on a Command key runs the Command, so never get wider than the keys you mean to read.
    fn get<'a>(
        &'a self,
        key_expression: &'a str,
        body: Option<QueryBody>,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<Reply>, CommsError>>;

    /// Keeps `key` alive until the returned token is dropped.
    fn declare_liveliness<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessToken, CommsError>>;

    /// Receives every liveliness change on a key that intersects `key_expression`, until the
    /// [`LivelinessSubscriber`] is dropped.
    fn subscribe_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessSubscriber, CommsError>>;

    /// Returns, within `timeout`, the keys of liveliness tokens that are alive.
    fn get_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<String>, CommsError>>;
}

/// The queries a queryable receives, in arrival order.
pub struct Queryable {
    queries: BoxStream<'static, Query>,
}

/// The samples a subscription receives, in publication order.
pub struct Subscriber {
    samples: BoxStream<'static, Sample>,
}

/// One get, as a queryable receives it. Dropping it without a reply tells the getter this queryable has nothing
/// to say.
pub struct Query {
    declared_key: String,
    key_expression: String,
    body: Option<QueryBody>,
    responder: Responder,
}

/// One publication: a key, an encoded payload and its metadata.
#[derive(Clone, Debug)]
pub struct Sample {
    key: String,
    payload: Payload,
    encoding: String,
    timestamp: Option<SystemTime>,
    attachment: Option<Payload>,
}

/// What a get sends along: an encoded payload, such as a Command's Message.
#[derive(Clone, Debug)]
pub struct QueryBody {
    payload: Payload,
    encoding: String,
}

/// An error a queryable replied instead of a sample, such as a body it could not decode.
#[derive(Clone, Debug)]
pub struct ReplyError {
    payload: Payload,
    encoding: String,
}

/// The bytes of a sample, a query or a reply. Cloning it shares the buffer, never copies it.
#[derive(Clone, Debug)]
pub struct Payload {
    buffer: Arc<dyn PayloadBuffer>,
}

/// A backend's own byte buffer, held inside a [`Payload`] so that it reaches the receiver untouched.
pub trait PayloadBuffer: Any + Debug + Send + Sync {
    /// The bytes, borrowed when the buffer is contiguous and copied only when it is not.
    fn to_bytes(&self) -> Cow<'_, [u8]>;
}

/// The liveliness changes a subscription receives, in arrival order.
pub struct LivelinessSubscriber {
    events: BoxStream<'static, LivelinessEvent>,
}

/// A liveliness token appeared or disappeared on the backbone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LivelinessEvent {
    /// A token was declared on `key`.
    Put {
        /// The key that became alive.
        key: String,
    },
    /// A token was dropped on `key`.
    Delete {
        /// The key that is no longer alive.
        key: String,
    },
}

/// Holds a liveliness declaration until it is dropped.
pub struct LivelinessToken {
    on_drop: Option<Box<dyn FnOnce() + Send>>,
}

/// Why the backbone refused an operation.
#[derive(Debug, thiserror::Error)]
pub enum CommsError {
    /// The key or key expression is not a valid canonical Zenoh key expression.
    #[error("invalid key expression {key_expression:?}")]
    InvalidKeyExpression {
        /// The rejected text.
        key_expression: String,
        /// What makes it invalid.
        #[source]
        source: Box<dyn core::error::Error + Send + Sync>,
    },
    /// The session or router rejected an operation.
    #[error("backend: {source}")]
    Backend {
        /// What the backend reported.
        #[source]
        source: Box<dyn core::error::Error + Send + Sync>,
    },
}

impl From<Bytes> for Payload {
    fn from(bytes: Bytes) -> Self {
        Self::new(bytes)
    }
}

impl From<Vec<u8>> for Payload {
    fn from(encoded: Vec<u8>) -> Self {
        Self::new(Bytes::from(encoded))
    }
}

impl Payload {
    /// Wraps a backend's buffer without copying it.
    pub fn new(buffer: impl PayloadBuffer) -> Self {
        Self {
            buffer: Arc::new(buffer),
        }
    }

    /// The bytes, borrowed when the buffer is contiguous and copied only when it is not.
    pub fn to_bytes(&self) -> Cow<'_, [u8]> {
        self.buffer.to_bytes()
    }

    /// The backend's own buffer, if it is a `T`, so a backend can send it on without a copy.
    pub fn downcast_ref<T: PayloadBuffer>(&self) -> Option<&T> {
        let buffer: &dyn Any = &*self.buffer;
        buffer.downcast_ref()
    }
}

impl PayloadBuffer for Bytes {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self)
    }
}

impl Sample {
    /// A sample with no timestamp and no attachment.
    pub fn new(
        key: impl Into<String>,
        payload: impl Into<Payload>,
        encoding: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            payload: payload.into(),
            encoding: encoding.into(),
            timestamp: None,
            attachment: None,
        }
    }

    /// Adds side-band metadata, such as a correlation id, so that the payload itself stays unframed.
    #[must_use]
    pub fn with_attachment(self, attachment: impl Into<Payload>) -> Self {
        Self {
            attachment: Some(attachment.into()),
            ..self
        }
    }

    /// Stamps the sample with the time its publisher gives it.
    #[must_use]
    pub fn with_timestamp(self, timestamp: SystemTime) -> Self {
        Self {
            timestamp: Some(timestamp),
            ..self
        }
    }

    /// The key it was published on.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The encoded bytes.
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// How the payload is encoded, for example `application/cdr;blueos_msgs/msg/ServiceInfo`.
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// When it was published, if the publisher or the router stamped it.
    pub fn timestamp(&self) -> Option<SystemTime> {
        self.timestamp
    }

    /// Side-band metadata, if the publisher attached any.
    pub fn attachment(&self) -> Option<&Payload> {
        self.attachment.as_ref()
    }
}

impl QueryBody {
    /// A body of `payload` encoded as `encoding`.
    pub fn new(payload: impl Into<Payload>, encoding: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
            encoding: encoding.into(),
        }
    }

    /// The encoded bytes.
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// How the payload is encoded.
    pub fn encoding(&self) -> &str {
        &self.encoding
    }
}

impl ReplyError {
    /// An error encoded as `encoding`.
    pub fn new(payload: impl Into<Payload>, encoding: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
            encoding: encoding.into(),
        }
    }

    /// The encoded error.
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// How the error is encoded.
    pub fn encoding(&self) -> &str {
        &self.encoding
    }
}

impl Query {
    /// A query that a backend delivers to the queryable declared on `declared_key`. `responder` carries the
    /// one reply back to the getter.
    pub fn new(
        declared_key: impl Into<String>,
        key_expression: impl Into<String>,
        body: Option<QueryBody>,
        responder: impl FnOnce(Reply) -> BoxFuture<'static, Result<(), CommsError>> + Send + 'static,
    ) -> Self {
        Self {
            declared_key: declared_key.into(),
            key_expression: key_expression.into(),
            body,
            responder: Box::new(responder),
        }
    }

    /// The key expression the getter asked for, which may hold wildcards.
    pub fn key_expression(&self) -> &str {
        &self.key_expression
    }

    /// What the getter sent along, if anything.
    pub fn body(&self) -> Option<&QueryBody> {
        self.body.as_ref()
    }

    /// Answers with a sample on the queryable's declared key.
    pub fn reply(
        self,
        payload: impl Into<Payload>,
        encoding: impl Into<String>,
    ) -> BoxFuture<'static, Result<(), CommsError>> {
        (self.responder)(Ok(Sample::new(self.declared_key, payload, encoding)))
    }

    /// Answers with an error instead of a sample.
    pub fn reply_error(
        self,
        payload: impl Into<Payload>,
        encoding: impl Into<String>,
    ) -> BoxFuture<'static, Result<(), CommsError>> {
        (self.responder)(Err(ReplyError {
            payload: payload.into(),
            encoding: encoding.into(),
        }))
    }
}

impl Subscriber {
    /// Wraps a backend's stream of samples.
    pub fn new(samples: impl Stream<Item = Sample> + Send + 'static) -> Self {
        Self {
            samples: samples.boxed(),
        }
    }

    /// Waits for the next sample, or returns `None` once the backend has closed the subscription.
    pub async fn recv(&mut self) -> Option<Sample> {
        self.samples.next().await
    }
}

impl Queryable {
    /// Wraps a backend's stream of queries.
    pub fn new(queries: impl Stream<Item = Query> + Send + 'static) -> Self {
        Self {
            queries: queries.boxed(),
        }
    }

    /// Waits for the next query, or returns `None` once the backend has closed the queryable.
    pub async fn recv(&mut self) -> Option<Query> {
        self.queries.next().await
    }
}

impl LivelinessSubscriber {
    /// Wraps a backend's stream of liveliness events.
    pub fn new(events: impl Stream<Item = LivelinessEvent> + Send + 'static) -> Self {
        Self {
            events: events.boxed(),
        }
    }

    /// Waits for the next event, or returns `None` once the backend has closed the subscription.
    pub async fn recv(&mut self) -> Option<LivelinessEvent> {
        self.events.next().await
    }
}

impl Drop for LivelinessToken {
    fn drop(&mut self) {
        if let Some(on_drop) = self.on_drop.take() {
            on_drop();
        }
    }
}

impl LivelinessToken {
    /// A token that runs `on_drop` when dropped.
    pub fn new(on_drop: impl FnOnce() + Send + 'static) -> Self {
        Self {
            on_drop: Some(Box::new(on_drop)),
        }
    }
}

impl CommsError {
    /// The session or router rejected an operation with `source`.
    pub fn backend(source: impl Into<Box<dyn core::error::Error + Send + Sync>>) -> Self {
        Self::Backend {
            source: source.into(),
        }
    }
}
