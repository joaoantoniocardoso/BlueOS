//! An in-process backbone for tests: every backend handle shared through one `Arc` sees the same bus.

use core::time::Duration;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex, PoisonError},
};

use futures_util::{FutureExt, future::BoxFuture, stream};
use tokio::{
    sync::mpsc,
    time::{Instant, timeout_at},
};
use zenoh_keyexpr::{OwnedKeyExpr, keyexpr};

use crate::{
    CommsBackend, CommsError, LivelinessEvent, LivelinessSubscriber, LivelinessToken, Query,
    QueryBody, Queryable, Reply, Sample, Subscriber,
};

/// Matches the Zenoh driver's `FifoChannel` depth: slow consumers lose samples instead of blocking publishers.
const HANDLER_CAPACITY: usize = 256;

/// A [`CommsBackend`] that delivers within the process, matching key expressions exactly as Zenoh does.
/// Default in-process backend for tests.
pub struct ChannelBackend {
    routes: Arc<Mutex<Routes>>,
}

impl Default for ChannelBackend {
    fn default() -> Self {
        Self {
            routes: Arc::new(Mutex::new(Routes::default())),
        }
    }
}

#[derive(Default)]
struct Routes {
    subscribers: Vec<Route<Sample>>,
    queryables: Vec<Route<Query>>,
    liveliness_keys: HashSet<OwnedKeyExpr>,
    liveliness_subscribers: Vec<Route<LivelinessEvent>>,
}

struct Route<T> {
    key_expression: OwnedKeyExpr,
    sender: mpsc::Sender<T>,
}

impl CommsBackend for ChannelBackend {
    fn publish(&self, sample: Sample) -> BoxFuture<'_, Result<(), CommsError>> {
        Box::pin(async move {
            let key = parse(sample.key())?;
            let mut routes = self.routes.lock().unwrap_or_else(PoisonError::into_inner);
            routes.subscribers.retain(|subscriber| {
                if !subscriber.key_expression.intersects(key) {
                    return !subscriber.sender.is_closed();
                }
                subscriber.sender.try_send(sample.clone()).is_ok() || !subscriber.sender.is_closed()
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
            let (sender, mut receiver) = mpsc::channel(HANDLER_CAPACITY);
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
            let (sender, mut receiver) = mpsc::channel(HANDLER_CAPACITY);
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
                            drop(reply_sender.send(reply));
                            async { Ok(()) }.boxed()
                        }
                    };
                    let query = Query::new(
                        queryable.key_expression.as_str(),
                        key_expression,
                        body.as_ref().cloned(),
                        responder,
                    );
                    queryable.sender.try_send(query).is_ok() || !queryable.sender.is_closed()
                });
            drop(reply_sender);
            let mut replies = Vec::new();
            while let Ok(Some(reply)) = timeout_at(deadline, reply_receiver.recv()).await {
                replies.push(reply);
            }
            Ok(replies)
        })
    }

    fn declare_liveliness<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessToken, CommsError>> {
        Box::pin(async move {
            let key_expression = parse(key)?.to_owned();
            let key_text = key_expression.to_string();
            let mut routes_guard = self.routes.lock().unwrap_or_else(PoisonError::into_inner);
            routes_guard.liveliness_keys.insert(key_expression.clone());
            notify_liveliness(
                &mut routes_guard,
                &key_expression,
                LivelinessEvent::Put {
                    key: key_text.clone(),
                },
            );
            let shared_routes = Arc::clone(&self.routes);
            Ok(LivelinessToken::new(move || {
                let mut drop_routes = shared_routes.lock().unwrap_or_else(PoisonError::into_inner);
                if drop_routes.liveliness_keys.remove(&key_expression) {
                    notify_liveliness(
                        &mut drop_routes,
                        &key_expression,
                        LivelinessEvent::Delete {
                            key: key_text.clone(),
                        },
                    );
                }
            }))
        })
    }

    fn subscribe_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessSubscriber, CommsError>> {
        Box::pin(async move {
            let key_expression = parse(key_expression)?.to_owned();
            let (sender, mut receiver) = mpsc::channel(HANDLER_CAPACITY);
            let replay = {
                let routes = self.routes.lock().unwrap_or_else(PoisonError::into_inner);
                routes
                    .liveliness_keys
                    .iter()
                    .filter(|key| key_expression.intersects(key))
                    .map(|key| LivelinessEvent::Put {
                        key: key.as_str().to_owned(),
                    })
                    .collect::<Vec<_>>()
            };
            for event in replay {
                let _ = sender.try_send(event);
            }
            self.routes
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .liveliness_subscribers
                .push(Route {
                    key_expression,
                    sender,
                });
            Ok(LivelinessSubscriber::new(stream::poll_fn(move |context| {
                receiver.poll_recv(context)
            })))
        })
    }

    fn get_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
        _timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<String>, CommsError>> {
        Box::pin(async move {
            let expression = parse(key_expression)?;
            let routes = self.routes.lock().unwrap_or_else(PoisonError::into_inner);
            Ok(routes
                .liveliness_keys
                .iter()
                .filter(|key| expression.intersects(key))
                .map(|key| key.as_str().to_owned())
                .collect())
        })
    }
}

fn notify_liveliness(routes: &mut Routes, key: &keyexpr, event: LivelinessEvent) {
    routes.liveliness_subscribers.retain(|subscriber| {
        if !subscriber.key_expression.intersects(key) {
            return !subscriber.sender.is_closed();
        }
        subscriber.sender.try_send(event.clone()).is_ok() || !subscriber.sender.is_closed()
    });
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
