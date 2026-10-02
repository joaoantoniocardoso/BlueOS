//! An in-process backbone for tests: every backend handle shared through one `Arc` sees the same bus.

use core::time::Duration;
use std::sync::{Mutex, PoisonError};

use futures_util::{FutureExt, future::BoxFuture, stream};
use tokio::{
    sync::mpsc,
    time::{Instant, timeout_at},
};
use zenoh_keyexpr::{OwnedKeyExpr, keyexpr};

use crate::{CommsBackend, CommsError, Query, QueryBody, Queryable, Reply, Sample, Subscriber};

/// A [`CommsBackend`] that delivers within the process, matching key expressions exactly as Zenoh does.
///
/// Delivery is unbounded, so a test never loses a sample or blocks a publisher.
#[derive(Default)]
pub struct ChannelBackend {
    routes: Mutex<Routes>,
}

#[derive(Default)]
struct Routes {
    subscribers: Vec<Route<Sample>>,
    queryables: Vec<Route<Query>>,
}

struct Route<T> {
    key_expression: OwnedKeyExpr,
    sender: mpsc::UnboundedSender<T>,
}

impl CommsBackend for ChannelBackend {
    fn publish(&self, sample: Sample) -> BoxFuture<'_, Result<(), CommsError>> {
        Box::pin(async move {
            let key = parse(sample.key())?;
            let mut routes = self.routes.lock().unwrap_or_else(PoisonError::into_inner);
            routes.subscribers.retain(|subscriber| {
                !subscriber.key_expression.intersects(key)
                    || subscriber.sender.send(sample.clone()).is_ok()
            });
            Ok(())
        })
    }

    fn subscribe<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<Subscriber, CommsError>> {
        Box::pin(async move {
            let key_expression = parse(key_expression)?.to_owned();
            let (sender, mut receiver) = mpsc::unbounded_channel();
            self.routes
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .subscribers
                .push(Route {
                    key_expression,
                    sender,
                });
            Ok(Subscriber::new(stream::poll_fn(move |context| {
                receiver.poll_recv(context)
            })))
        })
    }

    fn declare_queryable<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<Queryable, CommsError>> {
        Box::pin(async move {
            let key_expression = parse(key)?.to_owned();
            let (sender, mut receiver) = mpsc::unbounded_channel();
            self.routes
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .queryables
                .push(Route {
                    key_expression,
                    sender,
                });
            Ok(Queryable::new(stream::poll_fn(move |context| {
                receiver.poll_recv(context)
            })))
        })
    }

    fn get<'a>(
        &'a self,
        key_expression: &'a str,
        body: Option<QueryBody>,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<Reply>, CommsError>> {
        Box::pin(async move {
            let asked = parse(key_expression)?;
            let deadline = Instant::now() + timeout;
            let (reply_sender, mut reply_receiver) = mpsc::unbounded_channel();
            self.routes
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .queryables
                .retain(|queryable| {
                    if !queryable.key_expression.intersects(asked) {
                        return !queryable.sender.is_closed();
                    }
                    let responder = {
                        let reply_sender = mpsc::UnboundedSender::clone(&reply_sender);
                        move |reply| {
                            // A getter that timed out no longer listens; Zenoh drops such a reply silently too.
                            drop(reply_sender.send(reply));
                            async { Ok(()) }.boxed()
                        }
                    };
                    let query = Query::new(
                        queryable.key_expression.as_str(),
                        key_expression,
                        body.clone(),
                        responder,
                    );
                    queryable.sender.send(query).is_ok()
                });
            // Every query holds a sender, so the receiver closes once each queryable has replied or dropped it.
            drop(reply_sender);
            let mut replies = Vec::new();
            while let Ok(Some(reply)) = timeout_at(deadline, reply_receiver.recv()).await {
                replies.push(reply);
            }
            Ok(replies)
        })
    }
}

fn parse(key_expression: &str) -> Result<&keyexpr, CommsError> {
    keyexpr::new(key_expression).map_err(|error| CommsError::InvalidKeyExpression {
        key_expression: key_expression.to_owned(),
        source: error,
    })
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[tokio::test]
    async fn a_dropped_subscriber_is_removed_on_the_next_publish() {
        let backend = ChannelBackend::default();
        let kept = backend.subscribe("blueos/**").await.unwrap();
        drop(backend.subscribe("blueos/**").await.unwrap());

        backend
            .publish(Sample::new(
                "blueos/v1/example/log",
                Bytes::new(),
                "text/plain",
            ))
            .await
            .unwrap();

        assert_eq!(
            backend
                .routes
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .subscribers
                .len(),
            1
        );
        drop(kept);
    }
}
