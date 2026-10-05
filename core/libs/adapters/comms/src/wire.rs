//! Wire types and inherent implementations for backbone publish, subscribe, and query.

use core::{any::Any, fmt::Debug};
use std::{borrow::Cow, sync::Arc, time::SystemTime};

use bytes::Bytes;
use futures_util::{Stream, StreamExt, future::BoxFuture, stream::BoxStream};

/// A query handler's answer: a [`Sample`] on success or a [`ReplyError`] on failure.
pub type Reply = Result<Sample, ReplyError>;
pub(crate) type Responder =
    Box<dyn FnOnce(Reply) -> BoxFuture<'static, Result<(), CommsError>> + Send>;

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

#[derive(Clone, Debug, Default)]
struct SampleMetadata {
    timestamp: Option<SystemTime>,
    attachment: Option<Payload>,
}

/// One publication: a key, an encoded payload and its metadata.
#[derive(Clone, Debug)]
pub struct Sample {
    key: String,
    payload: Payload,
    encoding: String,
    metadata: SampleMetadata,
}

/// What a get sends along: an encoded payload, such as a Command's Message, and its metadata.
#[derive(Clone, Debug)]
pub struct QueryBody {
    payload: Payload,
    encoding: String,
    attachment: Option<Payload>,
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

    /// How many bytes the buffer holds, read without copying it.
    fn size_bytes(&self) -> usize;
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

// qual:allow(dry, boilerplate) reason: "From forwards into Payload::new for Bytes buffers"
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

    /// How many bytes the payload holds, read without copying it.
    pub fn size_bytes(&self) -> usize {
        self.buffer.size_bytes()
    }

    /// The backend's own buffer, if it is a `T`, so a backend can send it on without a copy.
    // qual:allow(coupling, deh) reason: "Payload downcast is the zero-copy seam for backend buffers"
    pub fn downcast_ref<T: PayloadBuffer>(&self) -> Option<&T> {
        let buffer: &dyn Any = &*self.buffer;
        buffer.downcast_ref()
    }
}

impl PayloadBuffer for Bytes {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self)
    }

    fn size_bytes(&self) -> usize {
        self.len()
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
            metadata: SampleMetadata::default(),
        }
    }

    /// Adds side-band metadata, such as a correlation id, so that the payload itself stays unframed.
    #[must_use]
    pub fn with_attachment(self, attachment: impl Into<Payload>) -> Self {
        Self {
            metadata: SampleMetadata {
                attachment: Some(attachment.into()),
                ..self.metadata
            },
            ..self
        }
    }

    /// Stamps the sample with the time its publisher gives it.
    #[must_use]
    pub fn with_timestamp(self, timestamp: SystemTime) -> Self {
        Self {
            metadata: SampleMetadata {
                timestamp: Some(timestamp),
                ..self.metadata
            },
            ..self
        }
    }

    /// The key it was published on.
    // qual:allow(dry, boilerplate) reason: "private fields: samples are built only through new/with_*"
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The encoded bytes.
    // qual:allow(dry, boilerplate) reason: "private fields: payload stays behind the Payload newtype"
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// How the payload is encoded, for example `application/cdr;blueos_msgs/msg/ServiceInfo`.
    // qual:allow(dry, boilerplate) reason: "private fields: encoding is fixed at construction time"
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// When it was published, if the publisher or the router stamped it.
    pub fn timestamp(&self) -> Option<SystemTime> {
        self.metadata.timestamp
    }

    /// Side-band metadata, if the publisher attached any.
    pub fn attachment(&self) -> Option<&Payload> {
        self.metadata.attachment.as_ref()
    }
}

impl QueryBody {
    /// A body of `payload` encoded as `encoding`, with no attachment.
    pub fn new(payload: impl Into<Payload>, encoding: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
            encoding: encoding.into(),
            attachment: None,
        }
    }

    /// Adds side-band metadata, such as the Job id a Command names, so that the payload itself stays unframed.
    #[must_use]
    pub fn with_attachment(self, attachment: impl Into<Payload>) -> Self {
        Self {
            attachment: Some(attachment.into()),
            ..self
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

    /// Side-band metadata, if the getter attached any.
    pub fn attachment(&self) -> Option<&Payload> {
        self.attachment.as_ref()
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
