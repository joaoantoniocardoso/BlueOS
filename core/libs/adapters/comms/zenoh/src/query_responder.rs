use std::sync::Mutex;

use crate::payload::zbytes_from_payload;
use async_trait::async_trait;
use blueos_comms_driver::{CommsError, Payload, QueryResponder, Result};

pub struct ZenohQueryResponder {
    pub key: String,
    pub query: Mutex<Option<zenoh::query::Query>>,
}

#[async_trait]
impl QueryResponder for ZenohQueryResponder {
    async fn reply(self: Box<Self>, payload: Payload, encoding: &str) -> Result<()> {
        let key = self.key.clone();
        let query = {
            let mut guard = self
                .query
                .lock()
                .map_err(|error| CommsError::Message(error.to_string()))?;
            guard.take().ok_or(CommsError::Closed)?
        };
        query
            .reply(key, zbytes_from_payload(payload))
            .encoding(encoding)
            .await
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        Ok(())
    }

    async fn reply_error(self: Box<Self>, message: &str) -> Result<()> {
        let query = {
            let mut guard = self
                .query
                .lock()
                .map_err(|error| CommsError::Message(error.to_string()))?;
            guard.take().ok_or(CommsError::Closed)?
        };
        query
            .reply_err(message)
            .await
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        Ok(())
    }
}
