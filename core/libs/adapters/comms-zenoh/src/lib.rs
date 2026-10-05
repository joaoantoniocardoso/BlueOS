//! Connects a service to the backbone through the local Zenoh router, passing each [`Payload`] on without a copy
//! where it can.

// Proving zenoh's `Session` is `Send` and `Sync` nests deeper than the default limit of 128.
#![recursion_limit = "256"]

pub mod config;
mod payload;
mod zenoh_backend;

use core::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use std::time::SystemTime;

use blueos_comms::{CommsError, Reply, ReplyError, Sample};
use futures_util::Stream;
use payload::payload_from_zbytes;
use tokio::sync::mpsc;
use zenoh_keyexpr::keyexpr;

/// Matches the in-process channel backend queue depth (D-10, findings #45).
pub(crate) const HANDLER_CAPACITY: usize = 256;

/// A [`CommsBackend`](blueos_comms::CommsBackend) backed by a Zenoh client session to `zenohd`.
pub struct ZenohBackend {
    session: zenoh::Session,
}

impl ZenohBackend {
    /// Opens a client session to `endpoint` (for example `tcp/127.0.0.1:7447`).
    // qual:api
    pub async fn connect(endpoint: &str) -> Result<Self, CommsError> {
        Self::connect_with_options(&config::ZenohConnectOptions {
            endpoint,
            config_file: None,
            zenoh_sets: &[],
        })
        .await
    }

    /// Opens a client session from the common CLI options (D-25).
    pub async fn connect_with_options(
        options: &config::ZenohConnectOptions<'_>,
    ) -> Result<Self, CommsError> {
        let configuration = config::client_config_from_options(options)?;
        Self::from_config(configuration).await
    }

    async fn from_config(configuration: zenoh::Config) -> Result<Self, CommsError> {
        let session = zenoh::open(configuration)
            .await
            .map_err(CommsError::backend)?;
        Ok(Self { session })
    }
}

pub(crate) struct ReceiverStream<T> {
    receiver: mpsc::Receiver<T>,
}

#[derive(Debug, thiserror::Error)]
#[error("the query was already answered")]
pub(crate) struct QueryAlreadyAnswered;

impl<T> Stream for ReceiverStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(context)
    }
}

pub(crate) fn sample_to_comms(sample: zenoh::sample::Sample) -> Sample {
    let mut comms = Sample::new(
        sample.key_expr().as_str(),
        payload_from_zbytes(sample.payload().clone()),
        sample.encoding().to_string(),
    );
    if let Some(timestamp) = sample.timestamp() {
        comms = comms.with_timestamp(zenoh_timestamp_to_system(timestamp));
    }
    if let Some(attachment) = sample.attachment() {
        comms = comms.with_attachment(payload_from_zbytes(attachment.clone()));
    }
    comms
}

pub(crate) fn reply_to_comms(reply: zenoh::query::Reply) -> Reply {
    match reply.into_result() {
        Ok(sample) => {
            let sample = sample_to_comms(sample);
            Ok(sample)
        }
        Err(error) => Err(ReplyError::new(
            payload_from_zbytes(error.payload().to_bytes().into()),
            error.encoding().to_string(),
        )),
    }
}

pub(crate) fn system_time_to_timestamp(
    session: &zenoh::Session,
    timestamp: SystemTime,
) -> zenoh::time::Timestamp {
    let duration = timestamp
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    zenoh::time::Timestamp::new(duration.into(), session.zid().into())
}

pub(crate) fn zenoh_timestamp_to_system(timestamp: &zenoh::time::Timestamp) -> SystemTime {
    SystemTime::UNIX_EPOCH + timestamp.get_time().to_duration()
}

pub(crate) fn parse(key_expression: &str) -> Result<&keyexpr, CommsError> {
    keyexpr::new(key_expression).map_err(|error| CommsError::InvalidKeyExpression {
        key_expression: key_expression.to_owned(),
        source: error,
    })
}
