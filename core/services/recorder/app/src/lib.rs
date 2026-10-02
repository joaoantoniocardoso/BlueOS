//! Recorder Service: data plane Task, MCAP files and the capture Domain.

#![expect(
    clippy::pub_use,
    reason = "the app crate re-exports Service entry types at the root"
)]

mod cli;
mod context;
mod data_plane;
pub mod endpoints;
mod service;
mod settings;

pub use cli::RecorderArguments;
pub use service::RecorderService;
