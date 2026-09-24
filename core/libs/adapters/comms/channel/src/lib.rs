//! In-process comms backend for hermetic tests (D-10).

mod broker;

use std::pin::Pin;
use std::time::Duration;

use async_trait::async_trait;
use blueos_comms_driver::{
    CommsBackend, LivelinessStream, LivelinessToken, Payload, QueryStream, Reply, Result,
    SampleStream,
};
use broker::{ChannelBroker, SharedBroker};
use bytes::Bytes;
use futures::Stream;
use tokio::sync::mpsc;

pub struct ChannelBackend {
    broker: SharedBroker,
}

impl Default for ChannelBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelBackend {
    pub fn new() -> Self {
        Self {
            broker: ChannelBroker::shared(),
        }
    }

    pub fn pair() -> (Self, Self) {
        let broker = ChannelBroker::shared();
        (
            Self {
                broker: broker.clone(),
            },
            Self { broker },
        )
    }
}

#[async_trait]
impl CommsBackend for ChannelBackend {
    async fn publish(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        attachment: Option<Payload>,
    ) -> Result<()> {
        self.broker
            .publish(key, payload, encoding, attachment)
            .await
    }

    async fn subscribe(&self, key_expression: &str) -> Result<SampleStream> {
        let receiver = self.broker.subscribe(key_expression).await?;
        Ok(Box::pin(ReceiverStream::new(receiver)))
    }

    async fn declare_queryable(&self, key_expression: &str) -> Result<QueryStream> {
        let receiver = self.broker.declare_queryable(key_expression).await?;
        Ok(Box::pin(ReceiverStream::new(receiver)))
    }

    async fn query(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        timeout: Duration,
    ) -> Result<Reply> {
        self.broker.query(key, payload, encoding, timeout).await
    }

    async fn declare_liveliness(&self, key: &str) -> Result<LivelinessToken> {
        self.broker.declare_liveliness(key).await
    }

    async fn subscribe_liveliness(&self, key_expression: &str) -> Result<LivelinessStream> {
        let receiver = self.broker.subscribe_liveliness(key_expression).await?;
        Ok(Box::pin(ReceiverStream::new(receiver)))
    }
}

struct ReceiverStream<T> {
    receiver: mpsc::Receiver<T>,
}

impl<T> ReceiverStream<T> {
    fn new(receiver: mpsc::Receiver<T>) -> Self {
        Self { receiver }
    }
}

impl<T> Stream for ReceiverStream<T> {
    type Item = T;

    fn poll_next(
        mut self: Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.receiver.poll_recv(context)
    }
}

pub fn payload_from_slice(data: &[u8]) -> Payload {
    Payload::from_bytes(Bytes::copy_from_slice(data))
}
