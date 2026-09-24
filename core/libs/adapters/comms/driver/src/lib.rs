//! Driver types and the async backend contract (D-10).
//!
//! Applications use [`blueos_comms::Session`]; this crate is shared by the Zenoh and in-process drivers.

mod key_match;
mod payload;

use std::pin::Pin;
use std::time::Duration;

use async_trait::async_trait;
use futures::Stream;
use thiserror::Error;

pub use key_match::key_matches;
pub use payload::{Payload, PayloadStorage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Endpoint {
    Local,
    Remote { url: String },
}

#[derive(Clone, Debug)]
pub struct Sample {
    /// Zenoh key expression for this sample.
    pub key: String,
    /// Sample body (may copy on read when backed by shared memory).
    pub payload: Payload,
    /// Wire encoding string (for example `application/cdr;...`).
    pub encoding: String,
    /// Optional publisher timestamp in nanoseconds since the Unix epoch.
    pub timestamp: Option<u64>,
    /// Optional attachment payload (type hash or side-band metadata).
    pub attachment: Option<Payload>,
}

#[derive(Clone, Debug)]
pub struct Reply {
    pub payload: Payload,
    pub encoding: String,
    pub attachment: Option<Payload>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LivelinessEvent {
    Put { key: String },
    Delete { key: String },
}

#[derive(Debug, Error)]
pub enum CommsError {
    #[error("{0}")]
    Message(String),
    #[error("zenoh: {0}")]
    Zenoh(String),
    #[error("query timed out after {0:?}")]
    QueryTimeout(Duration),
    #[error("no replier for query")]
    NoReplier,
    #[error("comms backend closed")]
    Closed,
}

pub type Result<T> = std::result::Result<T, CommsError>;

pub type SampleStream = Pin<Box<dyn Stream<Item = Sample> + Send>>;
pub type QueryStream = Pin<Box<dyn Stream<Item = IncomingQuery> + Send>>;
pub type LivelinessStream = Pin<Box<dyn Stream<Item = LivelinessEvent> + Send>>;

pub struct IncomingQuery {
    pub key: String,
    pub payload: Payload,
    pub encoding: String,
    pub attachment: Option<Payload>,
    responder: Box<dyn QueryResponder + Send>,
}

#[async_trait]
pub trait QueryResponder: Send {
    async fn reply(self: Box<Self>, payload: Payload, encoding: &str) -> Result<()>;
    async fn reply_error(self: Box<Self>, message: &str) -> Result<()>;
}

impl IncomingQuery {
    pub fn with_responder(
        key: String,
        payload: Payload,
        encoding: String,
        attachment: Option<Payload>,
        responder: Box<dyn QueryResponder + Send>,
    ) -> Self {
        Self {
            key,
            payload,
            encoding,
            attachment,
            responder,
        }
    }

    pub async fn reply(self, payload: Payload, encoding: &str) -> Result<()> {
        self.responder.reply(payload, encoding).await
    }

    pub async fn reply_error(self, message: &str) -> Result<()> {
        self.responder.reply_error(message).await
    }
}

pub struct ChannelQueryResponder {
    pub sender: tokio::sync::mpsc::Sender<ChannelQueryReply>,
}

pub enum ChannelQueryReply {
    Reply { payload: Payload, encoding: String },
    Error { message: String },
}

#[async_trait]
impl QueryResponder for ChannelQueryResponder {
    async fn reply(self: Box<Self>, payload: Payload, encoding: &str) -> Result<()> {
        self.sender
            .send(ChannelQueryReply::Reply {
                payload,
                encoding: encoding.to_string(),
            })
            .await
            .map_err(|_| CommsError::Closed)?;
        Ok(())
    }

    async fn reply_error(self: Box<Self>, message: &str) -> Result<()> {
        self.sender
            .send(ChannelQueryReply::Error {
                message: message.to_string(),
            })
            .await
            .map_err(|_| CommsError::Closed)?;
        Ok(())
    }
}

#[async_trait]
pub trait CommsBackend: Send + Sync {
    async fn publish(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        attachment: Option<Payload>,
    ) -> Result<()>;

    async fn subscribe(&self, key_expression: &str) -> Result<SampleStream>;

    async fn declare_queryable(&self, key_expression: &str) -> Result<QueryStream>;

    async fn query(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        timeout: Duration,
    ) -> Result<Reply>;

    async fn declare_liveliness(&self, key: &str) -> Result<LivelinessToken>;

    async fn subscribe_liveliness(&self, key_expression: &str) -> Result<LivelinessStream>;
}

pub struct LivelinessToken {
    drop: Box<dyn FnOnce() + Send>,
}

impl LivelinessToken {
    pub fn new(drop: Box<dyn FnOnce() + Send>) -> Self {
        Self { drop }
    }
}

impl Drop for LivelinessToken {
    fn drop(&mut self) {
        let drop = std::mem::replace(&mut self.drop, Box::new(|| {}));
        drop();
    }
}
