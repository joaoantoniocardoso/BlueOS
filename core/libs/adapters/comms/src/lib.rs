//! Async comms facade (D-10).
//!
//! Services open a [`Session`] as a Zenoh **client** to the local router (`tcp/127.0.0.1:7447` by default).
//! Payloads are not framed; use Zenoh attachments for extra metadata. [`Payload`] clones are cheap; with
//! shared memory enabled (default Zenoh config) large samples may use SHM when both peers support it.
//! Extensions need Docker `IpcMode: host` or a `/dev/shm` bind mount or SHM falls back to TCP (D-09).
//!
//! ## Example
//!
//! ```no_run
//! use std::time::Duration;
//!
//! use blueos_comms::{Endpoint, Payload, Session};
//! use bytes::Bytes;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), blueos_comms::CommsError> {
//!     let session = Session::open("my_service", Endpoint::Local).await?;
//!     let mut stream = session.subscribe("blueos/v1/my_service/**").await?;
//!     session
//!         .publish(
//!             "blueos/v1/my_service/events/ping",
//!             Payload::from_bytes(Bytes::from_static(b"hello")),
//!             "text/plain",
//!             None,
//!         )
//!         .await?;
//!     let _sample = tokio::time::timeout(Duration::from_secs(1), async {
//!         use futures::StreamExt;
//!         stream.next().await
//!     })
//!     .await;
//!     Ok(())
//! }
//! ```

mod state;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use blueos_comms_driver::{
    CommsBackend, LivelinessStream, LivelinessToken, QueryStream, Result, SampleStream,
};

#[cfg(feature = "channel")]
pub use blueos_comms_channel::ChannelBackend;

pub use blueos_comms_driver::{
    CommsError, Endpoint, IncomingQuery, LivelinessEvent, Payload, Reply, Sample,
};
pub use state::StateHandle;

enum Backend {
    #[cfg(feature = "zenoh")]
    Zenoh(blueos_comms_zenoh::ZenohBackend),
    #[cfg(feature = "channel")]
    Channel(blueos_comms_channel::ChannelBackend),
}

#[async_trait]
impl CommsBackend for Backend {
    async fn publish(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        attachment: Option<Payload>,
    ) -> Result<()> {
        match self {
            #[cfg(feature = "zenoh")]
            Backend::Zenoh(backend) => backend.publish(key, payload, encoding, attachment).await,
            #[cfg(feature = "channel")]
            Backend::Channel(backend) => backend.publish(key, payload, encoding, attachment).await,
        }
    }

    async fn subscribe(&self, key_expression: &str) -> Result<SampleStream> {
        match self {
            #[cfg(feature = "zenoh")]
            Backend::Zenoh(backend) => backend.subscribe(key_expression).await,
            #[cfg(feature = "channel")]
            Backend::Channel(backend) => backend.subscribe(key_expression).await,
        }
    }

    async fn declare_queryable(&self, key_expression: &str) -> Result<QueryStream> {
        match self {
            #[cfg(feature = "zenoh")]
            Backend::Zenoh(backend) => backend.declare_queryable(key_expression).await,
            #[cfg(feature = "channel")]
            Backend::Channel(backend) => backend.declare_queryable(key_expression).await,
        }
    }

    async fn query(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        timeout: Duration,
    ) -> Result<Reply> {
        match self {
            #[cfg(feature = "zenoh")]
            Backend::Zenoh(backend) => backend.query(key, payload, encoding, timeout).await,
            #[cfg(feature = "channel")]
            Backend::Channel(backend) => backend.query(key, payload, encoding, timeout).await,
        }
    }

    async fn declare_liveliness(&self, key: &str) -> Result<LivelinessToken> {
        match self {
            #[cfg(feature = "zenoh")]
            Backend::Zenoh(backend) => backend.declare_liveliness(key).await,
            #[cfg(feature = "channel")]
            Backend::Channel(backend) => backend.declare_liveliness(key).await,
        }
    }

    async fn subscribe_liveliness(&self, key_expression: &str) -> Result<LivelinessStream> {
        match self {
            #[cfg(feature = "zenoh")]
            Backend::Zenoh(backend) => backend.subscribe_liveliness(key_expression).await,
            #[cfg(feature = "channel")]
            Backend::Channel(backend) => backend.subscribe_liveliness(key_expression).await,
        }
    }
}

pub struct Session {
    backend: Arc<Backend>,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self {
            backend: Arc::clone(&self.backend),
        }
    }
}

impl Session {
    pub async fn open(service_name: &str, endpoint: blueos_comms_driver::Endpoint) -> Result<Self> {
        #[cfg(feature = "zenoh")]
        {
            let backend = blueos_comms_zenoh::ZenohBackend::open(service_name, endpoint).await?;
            Ok(Self {
                backend: Arc::new(Backend::Zenoh(backend)),
            })
        }
        #[cfg(not(feature = "zenoh"))]
        {
            let _ = (service_name, endpoint);
            Err(blueos_comms_driver::CommsError::Message(
                "blueos_comms built without feature zenoh".into(),
            ))
        }
    }

    #[cfg(feature = "channel")]
    pub fn with_channel(backend: blueos_comms_channel::ChannelBackend) -> Self {
        Self {
            backend: Arc::new(Backend::Channel(backend)),
        }
    }

    pub async fn publish(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        attachment: Option<Payload>,
    ) -> Result<()> {
        self.backend
            .publish(key, payload, encoding, attachment)
            .await
    }

    pub async fn subscribe(&self, key_expression: &str) -> Result<SampleStream> {
        self.backend.subscribe(key_expression).await
    }

    pub async fn declare_queryable(&self, key_expression: &str) -> Result<QueryStream> {
        self.backend.declare_queryable(key_expression).await
    }

    pub async fn query(
        &self,
        key: &str,
        payload: Payload,
        encoding: &str,
        timeout: Duration,
    ) -> Result<Reply> {
        self.backend.query(key, payload, encoding, timeout).await
    }

    pub async fn declare_liveliness(&self, key: &str) -> Result<LivelinessToken> {
        self.backend.declare_liveliness(key).await
    }

    pub async fn subscribe_liveliness(&self, key_expression: &str) -> Result<LivelinessStream> {
        self.backend.subscribe_liveliness(key_expression).await
    }

    pub async fn declare_state(&self, key: &str) -> Result<StateHandle> {
        state::declare_state(self.backend.clone(), key).await
    }
}
