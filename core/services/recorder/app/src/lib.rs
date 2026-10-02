//! Recorder Service: data plane Task, MCAP files and the capture Domain.

#![expect(
    clippy::pub_use,
    reason = "the app crate re-exports Service entry types at the root"
)]

mod cameras_io;
mod cli;
mod context;
mod data_plane;
pub mod endpoints;
mod handlers;
mod library_io;
mod mavlink;
mod sample_plan;
mod service;
mod settings;

pub use cli::RecorderArguments;
pub use service::{RecorderService, build_with_record_gate};
