#![expect(
    dead_code,
    reason = "Recording backend helpers are used when tests construct RecordingBackend"
)]

use core::{mem, sync::atomic::Ordering, time::Duration};
use std::sync::{Arc, Mutex, PoisonError};

use futures_util::{future::BoxFuture, stream};
use tokio::sync::Semaphore;

use blueos_comms::{
    CommsBackend, CommsError, LivelinessSubscriber, LivelinessToken, Query, QueryBody, Queryable,
    Reply, Sample, Subscriber,
};

use super::tank::{ClosedBackend, RecordingBackend, Sensor};

impl Default for Sensor {
    fn default() -> Self {
        Self(Arc::new(Semaphore::new(0)))
    }
}

impl CommsBackend for ClosedBackend {
    fn publish(&self, _sample: Sample) -> BoxFuture<'_, Result<(), CommsError>> {
        Box::pin(async { Ok(()) })
    }

    fn subscribe<'a>(
        &'a self,
        _key_expression: &'a str,
    ) -> BoxFuture<'a, Result<Subscriber, CommsError>> {
        Box::pin(async { Ok(Subscriber::new(stream::empty())) })
    }

    fn declare_queryable<'a>(
        &'a self,
        _key: &'a str,
    ) -> BoxFuture<'a, Result<Queryable, CommsError>> {
        Box::pin(async { Ok(Queryable::new(stream::empty())) })
    }

    fn get<'a>(
        &'a self,
        _key_expression: &'a str,
        _body: Option<QueryBody>,
        _timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<Reply>, CommsError>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn declare_liveliness<'a>(
        &'a self,
        _key: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessToken, CommsError>> {
        Box::pin(async { Ok(LivelinessToken::new(|| ())) })
    }

    fn subscribe_liveliness<'a>(
        &'a self,
        _key_expression: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessSubscriber, CommsError>> {
        Box::pin(async { Ok(LivelinessSubscriber::new(stream::empty())) })
    }

    fn get_liveliness<'a>(
        &'a self,
        _key_expression: &'a str,
        _timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<String>, CommsError>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}

impl CommsBackend for RecordingBackend {
    fn publish(&self, sample: Sample) -> BoxFuture<'_, Result<(), CommsError>> {
        if self.publishes_fail.load(Ordering::SeqCst) {
            return Box::pin(async {
                Err(CommsError::backend(std::io::Error::from(
                    std::io::ErrorKind::NotConnected,
                )))
            });
        }
        record(&self.journal, format!("publish {}", sample.key()));
        self.bus.publish(sample)
    }

    fn subscribe<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<Subscriber, CommsError>> {
        self.bus.subscribe(key_expression)
    }

    fn declare_queryable<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<Queryable, CommsError>> {
        Box::pin(async move {
            let queries = stream::unfold(self.bus.declare_queryable(key).await?, {
                let journal = Arc::clone(&self.journal);
                let key = key.to_owned();
                move |mut queryable| {
                    let journal = Arc::clone(&journal);
                    let key = key.clone();
                    async move {
                        let query = queryable.recv().await?;
                        Some((recorded(query, key, journal), queryable))
                    }
                }
            });
            Ok(Queryable::new(queries))
        })
    }

    fn get<'a>(
        &'a self,
        key_expression: &'a str,
        body: Option<QueryBody>,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<Reply>, CommsError>> {
        self.bus.get(key_expression, body, timeout)
    }

    fn declare_liveliness<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessToken, CommsError>> {
        self.bus.declare_liveliness(key)
    }

    fn subscribe_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessSubscriber, CommsError>> {
        self.bus.subscribe_liveliness(key_expression)
    }

    fn get_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<String>, CommsError>> {
        self.bus.get_liveliness(key_expression, timeout)
    }
}

impl RecordingBackend {
    /// Takes what was recorded so far.
    pub fn take_journal(&self) -> Vec<String> {
        mem::take(&mut self.journal.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

fn record(journal: &Arc<Mutex<Vec<String>>>, entry: String) {
    journal
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(entry);
}

fn recorded(query: Query, key: String, journal: Arc<Mutex<Vec<String>>>) -> Query {
    let key_expression = query.key_expression().to_owned();
    let body = query.body().cloned();
    Query::new(key.clone(), key_expression, body, move |reply: Reply| {
        record(&journal, format!("reply {key}"));
        let sample = reply.expect("the Kernel never replies with an error");
        query.reply(sample.payload().clone(), sample.encoding().to_owned())
    })
}
