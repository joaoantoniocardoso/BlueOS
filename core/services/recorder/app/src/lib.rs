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
pub use context::{
    IndexQuerySetup, IndexWalker, RepairBeforeRewrite, RepairIoSetup, default_index_walker,
};
pub use service::{
    RecorderService, build_with_record_gate, build_with_record_gate_and_index,
    build_with_record_gate_index_and_repair,
};
