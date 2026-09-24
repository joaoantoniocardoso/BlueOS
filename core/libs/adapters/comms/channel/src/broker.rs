use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use blueos_comms_driver::{
    ChannelQueryResponder, CommsError, IncomingQuery, LivelinessEvent, LivelinessToken, Payload,
    Reply, Result, Sample, key_matches,
};
use tokio::sync::mpsc;
use tracing::debug;

const CHANNEL_CAPACITY: usize = 64;

pub type SharedBroker = Arc<ChannelBroker>;

pub struct ChannelBroker {
    state: Arc<Mutex<BrokerState>>,
    next_id: AtomicU64,
}

struct BrokerState {
    subscribers: Vec<SubscriberEntry>,
    queryables: Vec<QueryableEntry>,
    liveliness_keys: HashMap<String, u64>,
    liveliness_subscribers: Vec<LivelinessSubscriberEntry>,
}

struct SubscriberEntry {
    key_expression: String,
    sender: mpsc::Sender<Sample>,
}

struct QueryableEntry {
    key_expression: String,
    sender: mpsc::Sender<IncomingQuery>,
}

struct LivelinessSubscriberEntry {
    key_expression: String,
    sender: mpsc::Sender<LivelinessEvent>,
}

impl ChannelBroker {
    pub fn new() -> SharedBroker {
        Arc::new(Self {
            state: Arc::new(Mutex::new(BrokerState {
                subscribers: Vec::new(),
                queryables: Vec::new(),
                liveliness_keys: HashMap::new(),
                liveliness_subscribers: Vec::new(),
            })),
            next_id: AtomicU64::new(1),
        })
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn publish(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        attachment: Option<Payload>,
    ) -> Result<()> {
        let sample = Sample {
            key: key.to_string(),
            payload,
            encoding: encoding.to_string(),
            timestamp: None,
            attachment,
        };
        let guard = self
            .state
            .lock()
            .map_err(|error| CommsError::Message(error.to_string()))?;
        for subscriber in &guard.subscribers {
            if key_matches(&subscriber.key_expression, key) {
                // ponytail: drop sample when subscriber queue is full (test driver backpressure)
                if let Err(error) = subscriber.sender.try_send(sample.clone()) {
                    debug!("channel publish dropped sample for {key}: {error}");
                }
            }
        }
        Ok(())
    }

    pub async fn subscribe(&self, key_expression: &str) -> Result<mpsc::Receiver<Sample>> {
        let (sender, receiver) = mpsc::channel(CHANNEL_CAPACITY);
        let mut guard = self
            .state
            .lock()
            .map_err(|error| CommsError::Message(error.to_string()))?;
        guard.subscribers.push(SubscriberEntry {
            key_expression: key_expression.to_string(),
            sender,
        });
        Ok(receiver)
    }

    pub async fn declare_queryable(
        &self,
        key_expression: &str,
    ) -> Result<mpsc::Receiver<IncomingQuery>> {
        let (sender, receiver) = mpsc::channel(CHANNEL_CAPACITY);
        let mut guard = self
            .state
            .lock()
            .map_err(|error| CommsError::Message(error.to_string()))?;
        guard.queryables.push(QueryableEntry {
            key_expression: key_expression.to_string(),
            sender,
        });
        Ok(receiver)
    }

    pub async fn query(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        timeout: Duration,
    ) -> Result<Reply> {
        let queryable_sender = {
            let guard = self
                .state
                .lock()
                .map_err(|error| CommsError::Message(error.to_string()))?;
            guard
                .queryables
                .iter()
                .find(|entry| key_matches(&entry.key_expression, key))
                .map(|entry| entry.sender.clone())
        };
        let Some(queryable_sender) = queryable_sender else {
            return Err(CommsError::NoReplier);
        };

        let (reply_sender, mut reply_receiver) = mpsc::channel(1);
        let query = IncomingQuery::with_responder(
            key.to_string(),
            payload,
            encoding.to_string(),
            None,
            Box::new(ChannelQueryResponder {
                sender: reply_sender,
            }),
        );
        if queryable_sender.send(query).await.is_err() {
            return Err(CommsError::NoReplier);
        }

        match tokio::time::timeout(timeout, reply_receiver.recv()).await {
            Ok(Some(blueos_comms_driver::ChannelQueryReply::Reply { payload, encoding })) => {
                Ok(Reply {
                    payload,
                    encoding,
                    attachment: None,
                })
            }
            Ok(Some(blueos_comms_driver::ChannelQueryReply::Error { message })) => {
                Err(CommsError::Message(message))
            }
            Ok(None) => Err(CommsError::NoReplier),
            Err(_) => Err(CommsError::QueryTimeout(timeout)),
        }
    }

    pub async fn declare_liveliness(&self, key: &str) -> Result<LivelinessToken> {
        let broker = Arc::new(self.clone());
        let key_owned = key.to_string();
        {
            let mut guard = self
                .state
                .lock()
                .map_err(|error| CommsError::Message(error.to_string()))?;
            guard
                .liveliness_keys
                .insert(key_owned.clone(), self.next_id());
            Self::notify_liveliness_locked(
                &guard,
                &key_owned,
                LivelinessEvent::Put {
                    key: key_owned.clone(),
                },
            );
        }
        let drop_key = key.to_string();
        Ok(LivelinessToken::new(Box::new(move || {
            if let Ok(mut guard) = broker.state.lock() {
                guard.liveliness_keys.remove(&drop_key);
                Self::notify_liveliness_locked(
                    &guard,
                    &drop_key,
                    LivelinessEvent::Delete {
                        key: drop_key.clone(),
                    },
                );
            }
        })))
    }

    pub async fn subscribe_liveliness(
        &self,
        key_expression: &str,
    ) -> Result<mpsc::Receiver<LivelinessEvent>> {
        let (sender, receiver) = mpsc::channel(CHANNEL_CAPACITY);
        let mut guard = self
            .state
            .lock()
            .map_err(|error| CommsError::Message(error.to_string()))?;
        let key_expression_owned = key_expression.to_string();
        guard
            .liveliness_subscribers
            .push(LivelinessSubscriberEntry {
                key_expression: key_expression_owned.clone(),
                sender: sender.clone(),
            });
        for key in guard.liveliness_keys.keys() {
            if key_matches(&key_expression_owned, key) {
                // ponytail: drop historical liveliness when subscriber queue is full
                if let Err(error) = sender.try_send(LivelinessEvent::Put { key: key.clone() }) {
                    debug!("channel liveliness replay dropped for {key}: {error}");
                }
            }
        }
        Ok(receiver)
    }

    fn notify_liveliness_locked(state: &BrokerState, key: &str, event: LivelinessEvent) {
        for subscriber in &state.liveliness_subscribers {
            if key_matches(&subscriber.key_expression, key) {
                // ponytail: drop liveliness event when subscriber queue is full
                if let Err(error) = subscriber.sender.try_send(event.clone()) {
                    debug!("channel liveliness notify dropped for {key}: {error}");
                }
            }
        }
    }
}

impl Clone for ChannelBroker {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            next_id: AtomicU64::new(self.next_id.load(Ordering::Relaxed)),
        }
    }
}
