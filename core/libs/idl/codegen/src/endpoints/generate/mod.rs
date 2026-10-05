//! Orchestrates endpoint discovery and emission (manifest → generated files).

mod manifest;
mod workspace;

pub use manifest::generate;
pub use workspace::{generate_all, stray_files};
