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
mod index_io;
mod io;
mod library_io;
mod library_observed;
mod mavlink;
mod sample_plan;
mod service;
mod settings;

pub use cli::RecorderArguments;
pub use context::{
    IndexQuerySetup, IndexWalker, RepairBeforeRewrite, RepairIoSetup, default_index_walker,
};
pub use service::{
    RecorderService, build_with_record_gate, build_with_record_gate_and_index,
    build_with_record_gate_index_and_repair,
};
