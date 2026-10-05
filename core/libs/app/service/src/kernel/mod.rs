//! The Kernel: the Inbox loop that applies every Command to the Domain and publishes what changed.

pub(crate) mod boot;
mod dispatch;
mod endpoints;
mod jobs;
mod publish;
mod run_loop;
mod start;
mod types;

mod effects;
pub(crate) mod io;
mod timers;

#[cfg(feature = "testing")]
pub(crate) use types::EffectLogStorage;
pub use types::Kernel;
pub(crate) use types::{Rejection, Unanswered};
