//! Recorder-specific command-line arguments.

use std::path::PathBuf;

/// Arguments for the Recorder Service.
#[derive(clap::Args, Clone, Debug)]
pub struct RecorderArguments {
    /// Directory where MCAP recordings are stored.
    #[arg(long, value_name = "DIR")]
    pub recorder_path: PathBuf,
}
