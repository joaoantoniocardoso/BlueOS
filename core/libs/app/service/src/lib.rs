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
mod clock;
pub mod entry;
mod kernel;
mod logging;
#[cfg(feature = "probe")]
pub mod probe;
mod run_outcome;
mod service;
mod settings;
mod shutdown;
mod tasks;
#[cfg(feature = "testing")]
pub mod testing;

pub use blueos_domain::IoError;
pub use builder::{Refusal, ServiceBuilder};
pub use clock::Clock;
pub use kernel::Kernel;
pub use run_outcome::RunOutcome;
pub use service::{Service, ServiceContext, ServiceError};
pub use shutdown::ShutdownHandle;
pub use tasks::{RestartPolicy, TaskContext, TaskFailed};
