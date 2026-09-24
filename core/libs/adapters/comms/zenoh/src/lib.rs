//! Zenoh driver: client mode to local `zenohd`, zero-copy [`Payload`] where possible (D-09, D-10).

mod config;
mod payload;
mod query_responder;

use std::pin::Pin;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use blueos_comms_driver::{
    CommsBackend, CommsError, Endpoint, IncomingQuery, LivelinessEvent, LivelinessStream,
    LivelinessToken, Payload, QueryStream, Reply, Result, Sample, SampleStream,
};
use futures::Stream;
use payload::{payload_from_zbytes, zbytes_from_payload};
use query_responder::ZenohQueryResponder;
use tokio::sync::mpsc;
use zenoh::Wait;
use zenoh::handlers::FifoChannel;
use zenoh::sample::SampleKind;

const HANDLER_CAPACITY: usize = 256;

pub struct ZenohBackend {
    session: zenoh::Session,
}

impl ZenohBackend {
    pub async fn open(service_name: &str, endpoint: Endpoint) -> Result<Self> {
        let configuration = config::load_config(service_name, endpoint)?;
        let session = zenoh::open(configuration)
            .await
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        Ok(Self { session })
    }

    pub fn session(&self) -> &zenoh::Session {
        &self.session
    }
}

#[async_trait]
impl CommsBackend for ZenohBackend {
    async fn publish(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        attachment: Option<Payload>,
    ) -> Result<()> {
        let mut builder = self
            .session
            .put(key, zbytes_from_payload(payload))
            .encoding(encoding);
        if let Some(attachment) = attachment {
            builder = builder.attachment(zbytes_from_payload(attachment));
        }
        builder
            .await
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        Ok(())
    }

    async fn subscribe(&self, key_expression: &str) -> Result<SampleStream> {
        let (sender, receiver) = mpsc::channel(HANDLER_CAPACITY);
        let subscriber = self
            .session
            .declare_subscriber(key_expression)
            .with(FifoChannel::new(HANDLER_CAPACITY))
            .await
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                let zenoh_sample = sample_to_comms(sample);
                if sender.send(zenoh_sample).await.is_err() {
                    break;
                }
            }
        });
        Ok(Box::pin(ReceiverStream { receiver }))
    }

    async fn declare_queryable(&self, key_expression: &str) -> Result<QueryStream> {
        let (sender, receiver) = mpsc::channel(HANDLER_CAPACITY);
        let queryable = self
            .session
            .declare_queryable(key_expression)
            .with(FifoChannel::new(HANDLER_CAPACITY))
            .await
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        tokio::spawn(async move {
            while let Ok(query) = queryable.recv_async().await {
                let key = query.key_expr().as_str().to_string();
                let encoding = query
                    .encoding()
                    .map(|value| value.to_string())
                    .unwrap_or_default();
                let payload = query
                    .payload()
                    .map(|value| payload_from_zbytes(value.to_bytes().into()))
                    .unwrap_or_else(Payload::empty);
                let attachment = query
                    .attachment()
                    .map(|value| payload_from_zbytes(value.to_bytes().into()));
                let incoming = IncomingQuery::with_responder(
                    key.clone(),
                    payload,
                    encoding,
                    attachment,
                    Box::new(ZenohQueryResponder {
                        key,
                        query: Mutex::new(Some(query)),
                    }),
                );
                if sender.send(incoming).await.is_err() {
                    break;
                }
            }
        });
        Ok(Box::pin(ReceiverStream { receiver }))
    }

    async fn query(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        timeout: Duration,
    ) -> Result<Reply> {
        let replies = self
            .session
            .get(key)
            .payload(zbytes_from_payload(payload))
            .encoding(encoding)
            .await
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        match tokio::time::timeout(timeout, replies.recv_async()).await {
            Ok(Ok(reply)) => match reply.into_result() {
                Ok(sample) => Ok(reply_sample_to_comms(sample)),
                Err(error) => Err(CommsError::Zenoh(error.to_string())),
            },
            Ok(Err(error)) => Err(CommsError::Zenoh(error.to_string())),
            Err(_) => Err(CommsError::QueryTimeout(timeout)),
        }
    }

    async fn declare_liveliness(&self, key: &str) -> Result<LivelinessToken> {
        let token = self
            .session
            .liveliness()
            .declare_token(key)
            .await
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        Ok(LivelinessToken::new(Box::new(move || {
            // Drop runs synchronously; Zenoh undeclare blocks the dropping thread.
            let _ = token.undeclare().wait();
        })))
    }

    async fn subscribe_liveliness(&self, key_expression: &str) -> Result<LivelinessStream> {
        let (sender, receiver) = mpsc::channel(HANDLER_CAPACITY);
        let subscriber = self
            .session
            .liveliness()
            .declare_subscriber(key_expression)
            .history(true)
            .with(FifoChannel::new(HANDLER_CAPACITY))
            .await
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                let event = match sample.kind() {
                    SampleKind::Put => LivelinessEvent::Put {
                        key: sample.key_expr().as_str().to_string(),
                    },
                    SampleKind::Delete => LivelinessEvent::Delete {
                        key: sample.key_expr().as_str().to_string(),
                    },
                };
                if sender.send(event).await.is_err() {
                    break;
                }
            }
        });
        Ok(Box::pin(ReceiverStream { receiver }))
    }
}

struct ReceiverStream<T> {
    receiver: mpsc::Receiver<T>,
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

fn sample_to_comms(sample: zenoh::sample::Sample) -> Sample {
    let timestamp = sample.timestamp().map(|value| value.get_time().as_u64());
    Sample {
        key: sample.key_expr().as_str().to_string(),
        payload: payload_from_zbytes(sample.payload().clone()),
        encoding: sample.encoding().to_string(),
        timestamp,
        attachment: sample
            .attachment()
            .map(|value| payload_from_zbytes(value.clone())),
    }
}

fn reply_sample_to_comms(sample: zenoh::sample::Sample) -> Reply {
    Reply {
        payload: payload_from_zbytes(sample.payload().clone()),
        encoding: sample.encoding().to_string(),
        attachment: sample
            .attachment()
            .map(|value| payload_from_zbytes(value.clone())),
    }
}

pub mod test_support {
    use super::*;

    pub struct TestRouter {
        _router: zenoh::Session,
        pub client: ZenohBackend,
        pub endpoint: String,
    }

    pub async fn open_test_router() -> Result<TestRouter> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .map_err(|error| CommsError::Message(error.to_string()))?;
        let port = listener
            .local_addr()
            .map_err(|error| CommsError::Message(error.to_string()))?
            .port();
        drop(listener);
        let endpoint = format!("tcp/127.0.0.1:{port}");
        let router_json = format!(
            r#"{{
            mode: "router",
            listen: {{ endpoints: ["{endpoint}"] }},
            scouting: {{ multicast: {{ enabled: false }} }}
        }}"#
        );
        let configuration = zenoh::Config::from_json5(&router_json)
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        let router = zenoh::open(configuration)
            .await
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        let connect_url = endpoint.clone();
        let client =
            ZenohBackend::open("test_router_client", Endpoint::Remote { url: connect_url }).await?;
        Ok(TestRouter {
            _router: router,
            client,
            endpoint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blueos_comms_driver::Payload;
    use bytes::Bytes;
    use futures::StreamExt;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "needs a local zenohd router"]
    async fn zenoh_publish_subscribe_round_trip() {
        let harness = test_support::open_test_router()
            .await
            .expect("open test router");
        let publisher = harness.client;
        let subscriber_backend = ZenohBackend::open(
            "test_subscriber",
            Endpoint::Remote {
                url: harness.endpoint.clone(),
            },
        )
        .await
        .expect("subscriber session");
        let mut stream = subscriber_backend
            .subscribe("test/topic")
            .await
            .expect("subscribe");
        tokio::time::sleep(Duration::from_millis(100)).await;
        publisher
            .publish(
                "test/topic",
                Payload::from_bytes(Bytes::from_static(b"hello")),
                "text/plain",
                None,
            )
            .await
            .expect("publish");
        let sample = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .expect("timeout")
            .expect("stream ended");
        assert_eq!(sample.payload.as_slice(), b"hello");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "needs a local zenohd router"]
    async fn zenoh_large_payload_round_trip() {
        let harness = test_support::open_test_router()
            .await
            .expect("open test router");
        let publisher = harness.client;
        let subscriber_backend = ZenohBackend::open(
            "test_subscriber_large",
            Endpoint::Remote {
                url: harness.endpoint.clone(),
            },
        )
        .await
        .expect("subscriber session");
        let mut stream = subscriber_backend
            .subscribe("test/large")
            .await
            .expect("subscribe");
        tokio::time::sleep(Duration::from_millis(100)).await;
        let large = vec![0xAB_u8; 4096];
        publisher
            .publish(
                "test/large",
                Payload::from_bytes(Bytes::from(large.clone())),
                "application/octet-stream",
                None,
            )
            .await
            .expect("publish");
        let sample = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .expect("timeout")
            .expect("stream ended");
        assert_eq!(sample.payload.as_slice(), large.as_slice());
    }
}
