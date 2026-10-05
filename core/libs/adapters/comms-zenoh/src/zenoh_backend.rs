use core::time::Duration;
use std::sync::{Arc, Mutex, PoisonError};

use futures_util::{FutureExt, future::BoxFuture};
use tokio::sync::mpsc;
use zenoh::{Wait, handlers::FifoChannel, query::ConsolidationMode, sample::SampleKind};

use blueos_comms::{
    CommsBackend, CommsError, LivelinessEvent, LivelinessSubscriber, LivelinessToken, Query,
    QueryBody, Queryable, Reply, Sample, Subscriber,
};

use crate::{
    HANDLER_CAPACITY, QueryAlreadyAnswered, ReceiverStream, ZenohBackend, parse, reply_to_comms,
    sample_to_comms, system_time_to_timestamp,
};

use super::payload::{payload_from_zbytes, zbytes_from_payload};

impl CommsBackend for ZenohBackend {
    fn publish(&self, sample: Sample) -> BoxFuture<'_, Result<(), CommsError>> {
        Box::pin(async move {
            parse(sample.key())?;
            let mut builder = self
                .session
                .put(sample.key(), zbytes_from_payload(sample.payload()))
                .encoding(sample.encoding());
            if let Some(timestamp) = sample.timestamp() {
                builder = builder.timestamp(system_time_to_timestamp(&self.session, timestamp));
            }
            if let Some(attachment) = sample.attachment() {
                builder = builder.attachment(zbytes_from_payload(attachment));
            }
            builder.await.map_err(CommsError::backend)?;
            Ok(())
        })
    }

    fn subscribe<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<Subscriber, CommsError>> {
        Box::pin(async move {
            parse(key_expression)?;
            let (sender, receiver) = mpsc::channel(HANDLER_CAPACITY);
            let subscriber = self
                .session
                .declare_subscriber(key_expression)
                .with(FifoChannel::new(HANDLER_CAPACITY))
                .await
                .map_err(CommsError::backend)?;
            tokio::spawn(async move {
                while let Ok(sample) = subscriber.recv_async().await {
                    let sample = sample_to_comms(sample);
                    if sender.send(sample).await.is_err() {
                        break;
                    }
                }
            });
            Ok(Subscriber::new(ReceiverStream { receiver }))
        })
    }

    fn declare_queryable<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<Queryable, CommsError>> {
        Box::pin(async move {
            parse(key)?;
            let (sender, receiver) = mpsc::channel(HANDLER_CAPACITY);
            let queryable = self
                .session
                .declare_queryable(key)
                .with(FifoChannel::new(HANDLER_CAPACITY))
                .await
                .map_err(CommsError::backend)?;
            let declared_key = Arc::new(key.to_string());
            tokio::spawn(async move {
                while let Ok(query) = queryable.recv_async().await {
                    let key_expression = query.key_expr().as_str().to_string();
                    let body = query.payload().map(|wire_payload| {
                        let encoding = query
                            .encoding()
                            .map(|wire_encoding| wire_encoding.to_string())
                            .unwrap_or_default();
                        let body = QueryBody::new(
                            payload_from_zbytes(wire_payload.to_bytes().into()),
                            encoding,
                        );
                        match query.attachment() {
                            Some(attachment) => body
                                .with_attachment(payload_from_zbytes(attachment.to_bytes().into())),
                            None => body,
                        }
                    });
                    let query = Mutex::new(Some(query));
                    let reply_key = declared_key.to_string();
                    let incoming = Query::new(
                        declared_key.to_string(),
                        key_expression,
                        body,
                        move |reply| {
                            let query = query;
                            let reply_declared_key = reply_key;
                            async move {
                                let query = query
                                    .lock()
                                    .unwrap_or_else(PoisonError::into_inner)
                                    .take()
                                    .ok_or(CommsError::backend(QueryAlreadyAnswered))?;
                                match reply {
                                    Ok(sample) => {
                                        let mut builder = query
                                            .reply(
                                                reply_declared_key,
                                                zbytes_from_payload(sample.payload()),
                                            )
                                            .encoding(sample.encoding());
                                        if let Some(attachment) = sample.attachment() {
                                            builder =
                                                builder.attachment(zbytes_from_payload(attachment));
                                        }
                                        builder.await.map_err(CommsError::backend)?;
                                    }
                                    Err(error) => {
                                        query
                                            .reply_err(zbytes_from_payload(error.payload()))
                                            .encoding(error.encoding())
                                            .await
                                            .map_err(CommsError::backend)?;
                                    }
                                }
                                Ok(())
                            }
                            .boxed()
                        },
                    );
                    if sender.send(incoming).await.is_err() {
                        break;
                    }
                }
            });
            Ok(Queryable::new(ReceiverStream { receiver }))
        })
    }

    fn get<'a>(
        &'a self,
        key_expression: &'a str,
        body: Option<QueryBody>,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<Reply>, CommsError>> {
        Box::pin(async move {
            parse(key_expression)?;
            let mut builder = self
                .session
                .get(key_expression)
                .consolidation(ConsolidationMode::None);
            if let Some(body) = body {
                builder = builder
                    .payload(zbytes_from_payload(body.payload()))
                    .encoding(body.encoding());
                if let Some(attachment) = body.attachment() {
                    builder = builder.attachment(zbytes_from_payload(attachment));
                }
            }
            let replies = builder.await.map_err(CommsError::backend)?;
            let deadline = tokio::time::Instant::now() + timeout;
            let mut collected = Vec::new();
            while tokio::time::Instant::now() < deadline {
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                match tokio::time::timeout(remaining, replies.recv_async()).await {
                    Ok(Ok(reply)) => collected.push(reply_to_comms(reply)),
                    _ => break,
                }
            }
            Ok(collected)
        })
    }

    fn declare_liveliness<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessToken, CommsError>> {
        Box::pin(async move {
            parse(key)?;
            let token = self
                .session
                .liveliness()
                .declare_token(key)
                .await
                .map_err(CommsError::backend)?;
            Ok(LivelinessToken::new(move || {
                let _ = token.undeclare().wait();
            }))
        })
    }

    fn subscribe_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessSubscriber, CommsError>> {
        Box::pin(async move {
            parse(key_expression)?;
            let (sender, receiver) = mpsc::channel(HANDLER_CAPACITY);
            let subscriber = self
                .session
                .liveliness()
                .declare_subscriber(key_expression)
                .history(true)
                .with(FifoChannel::new(HANDLER_CAPACITY))
                .await
                .map_err(CommsError::backend)?;
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
            Ok(LivelinessSubscriber::new(ReceiverStream { receiver }))
        })
    }

    fn get_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<String>, CommsError>> {
        Box::pin(async move {
            parse(key_expression)?;
            let replies = self
                .session
                .liveliness()
                .get(key_expression)
                .await
                .map_err(CommsError::backend)?;
            let deadline = tokio::time::Instant::now() + timeout;
            let mut keys = Vec::new();
            while tokio::time::Instant::now() < deadline {
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                match tokio::time::timeout(remaining, replies.recv_async()).await {
                    Ok(Ok(reply)) => {
                        if let Ok(sample) = reply.into_result() {
                            keys.push(sample.key_expr().as_str().to_string());
                        }
                    }
                    _ => break,
                }
            }
            keys.sort();
            keys.dedup();
            Ok(keys)
        })
    }
}
