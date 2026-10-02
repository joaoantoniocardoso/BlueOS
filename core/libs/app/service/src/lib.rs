//! Runs a BlueOS Service: the [`Service`] trait a service implements, the [`ServiceBuilder`] its `build` fills,
//! and the [`Kernel`] that runs its Domain.
//!
//! A service writes no async code and no wiring loop. `build` declares the Command endpoints and States from the
//! Domain's types; the Kernel puts every Request into one Inbox, applies each Command to the Domain as a
//! transaction, publishes the States, acknowledges the Command, and only then publishes the Events. Tests run the
//! same `build` through [`testing::Harness`].

#![expect(
    clippy::pub_use,
    reason = "the crate root exposes its own modules as one flat API"
)]

mod builder;
pub mod entry;
mod kernel;
#[cfg(feature = "probe")]
pub mod probe;
mod service;
#[cfg(feature = "testing")]
pub mod testing;

pub use builder::ServiceBuilder;
pub use kernel::{Clock, Kernel};
pub use service::{Service, ServiceContext, ServiceError};
