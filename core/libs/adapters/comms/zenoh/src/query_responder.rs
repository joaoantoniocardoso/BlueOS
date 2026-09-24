use std::sync::Mutex;

use async_trait::async_trait;
use blueos_comms_driver::{CommsError, Payload, QueryResponder, Result};
use zenoh::Wait;

use crate::payload::zbytes_from_payload;

pub struct ZenohQueryResponder {
    pub key: String,
    pub query: Mutex<Option<zenoh::query::Query>>,
}

#[async_trait]
impl QueryResponder for ZenohQueryResponder {
    async fn reply(self: Box<Self>, payload: Payload, encoding: &str) -> Result<()> {
        let mut guard = self
            .query
            .lock()
            .map_err(|error| CommsError::Message(error.to_string()))?;
        let query = guard.take().ok_or(CommsError::Closed)?;
        query
            .reply(self.key.clone(), zbytes_from_payload(payload))
            .encoding(encoding)
            .wait()
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        Ok(())
    }

    async fn reply_error(self: Box<Self>, message: &str) -> Result<()> {
        let mut guard = self
            .query
            .lock()
            .map_err(|error| CommsError::Message(error.to_string()))?;
        let query = guard.take().ok_or(CommsError::Closed)?;
        query
            .reply_err(message)
            .wait()
            .map_err(|error| CommsError::Zenoh(error.to_string()))?;
        Ok(())
    }
}
