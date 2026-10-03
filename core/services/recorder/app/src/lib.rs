//! Recorder Service: data plane Task, MCAP files and the capture Domain.

#![expect(
    clippy::pub_use,
    reason = "the app crate re-exports Service entry types at the root"
)]

mod cameras;
mod capture;
mod cli;
mod context;
pub mod endpoints;
mod io;
mod library;
mod service;
mod settings;
mod tasks;

pub use cli::RecorderArguments;
pub use context::{IndexWalker, RecorderContext, Rewriter};
pub use service::RecorderService;
