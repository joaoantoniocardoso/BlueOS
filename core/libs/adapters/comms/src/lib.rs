//! How a Service talks on the BlueOS backbone: publish, subscribe, answer queries and query others.
//!
//! The Kernel holds one `Arc<dyn CommsBackend>`: the Zenoh backend in a shipped binary, the in-process `channel`
//! backend in tests (its feature belongs in `[dev-dependencies]` only). Payloads are never framed or copied on the
//! way through; metadata travels in the attachment.

#![expect(
    clippy::pub_use,
    reason = "wire types are defined in wire.rs and exported at the crate root"
)]

#[cfg(feature = "channel")]
pub mod channel;

mod wire;

use core::time::Duration;

use futures_util::future::BoxFuture;

pub use wire::{
    CommsError, LivelinessEvent, LivelinessSubscriber, LivelinessToken, Payload, PayloadBuffer,
    Query, QueryBody, Queryable, Reply, ReplyError, Sample, Subscriber,
};

/// Publishing, subscribing and querying on the backbone. A Service shares one as `Arc<dyn CommsBackend>`; in a
/// shipped binary it is the Service's Session.
///
/// Key expressions follow Zenoh: `*` matches one chunk, `**` any number of chunks and `$*` part of a chunk. They
/// must be in canonical form (`a/**`, not `a/**/**`).
pub trait CommsBackend: Send + Sync {
    /// Sends `sample` to every subscriber whose key expression intersects its key.
    fn publish(&self, sample: Sample) -> BoxFuture<'_, Result<(), CommsError>>;

    /// Receives every sample published on a key that intersects `key_expression`, until the [`Subscriber`] is
    /// dropped.
    fn subscribe<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<Subscriber, CommsError>>;

    /// Receives every get whose key expression intersects `key`, until the [`Queryable`] is dropped. Replies
    /// always carry `key`, so a wildcard get tells them apart.
    fn declare_queryable<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<Queryable, CommsError>>;

    /// Sends a query to every queryable whose key intersects `key_expression` and returns their replies in
    /// arrival order, once each of them has replied or dropped its query, or when `timeout` expires.
    ///
    /// A get on a Command key runs the Command, so never get wider than the keys you mean to read.
    fn get<'a>(
        &'a self,
        key_expression: &'a str,
        body: Option<QueryBody>,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<Reply>, CommsError>>;

    /// Keeps `key` alive until the returned token is dropped.
    fn declare_liveliness<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessToken, CommsError>>;

    /// Receives every liveliness change on a key that intersects `key_expression`, until the
    /// [`LivelinessSubscriber`] is dropped.
    fn subscribe_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessSubscriber, CommsError>>;

    /// Returns, within `timeout`, the keys of liveliness tokens that are alive.
    fn get_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<String>, CommsError>>;
}
