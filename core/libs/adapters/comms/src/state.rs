use std::sync::Arc;

use blueos_comms_driver::{CommsBackend, Payload, Result};
use futures::StreamExt;
use tokio::sync::Mutex;

use super::Backend;

#[derive(Clone)]
struct StateSnapshot {
    payload: Payload,
    encoding: String,
}

pub struct StateHandle {
    key: String,
    backend: Arc<Backend>,
    snapshot: Arc<Mutex<Option<StateSnapshot>>>,
}

pub async fn declare_state(backend: Arc<Backend>, key: &str) -> Result<StateHandle> {
    let snapshot = Arc::new(Mutex::new(None));
    let key_owned = key.to_string();
    let mut query_stream = backend.declare_queryable(key).await?;
    let snapshot_for_query = Arc::clone(&snapshot);
    let key_for_query = key_owned.clone();
    tokio::spawn(async move {
        while let Some(query) = query_stream.next().await {
            let stored: Option<StateSnapshot> = snapshot_for_query.lock().await.clone();
            match stored {
                Some(value) => {
                    let _ = query.reply(value.payload, &value.encoding).await;
                }
                None => {
                    let _ = query.reply_error("state not initialized").await;
                }
            }
            let _ = key_for_query;
        }
    });
    Ok(StateHandle {
        key: key_owned,
        backend,
        snapshot,
    })
}

impl StateHandle {
    pub async fn publish(&self, payload: Payload, encoding: &str) -> Result<()> {
        {
            let mut guard = self.snapshot.lock().await;
            *guard = Some(StateSnapshot {
                payload: payload.clone(),
                encoding: encoding.to_string(),
            });
        }
        self.backend
            .publish(&self.key, payload, encoding, None)
            .await
    }
}
