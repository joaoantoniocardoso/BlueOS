//! Arguments every Service shares on the command line.

use std::path::PathBuf;

use clap::Args;

/// Arguments parsed for every Service before its own [`clap::Args`].
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub struct CommonArguments {
    /// Raise the log level: `-v` for debug, `-vv` for trace.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Folder that holds `<service>/settings-<N>.json`. Default: the user config folder.
    #[arg(long, value_name = "DIR")]
    pub settings_path: Option<PathBuf>,
    /// Zenoh router to connect to.
    #[arg(long, value_name = "ENDPOINT", default_value = "tcp/127.0.0.1:7447")]
    pub zenoh_endpoint: String,
    /// Full Zenoh JSON5 config file. Replaces `--zenoh-endpoint`.
    #[arg(long, value_name = "FILE", env = "ZENOH_CONFIG")]
    pub zenoh_config: Option<PathBuf>,
    /// Override one Zenoh config key, for example `--zenoh-set transport/shared_memory/enabled=false`.
    #[arg(long = "zenoh-set", value_name = "PATH=JSON5")]
    pub zenoh_sets: Vec<String>,
}
