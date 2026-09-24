use std::collections::HashMap;
use std::num::NonZeroU64;
use std::path::PathBuf;

use blueos_recorder_mcap::{
    DEFAULT_CHUNK_BYTES, DEFAULT_FLUSH_INTERVAL_SECS, McapCompression, McapWriteConfig,
};
use clap::Parser;
use tracing::warn;

#[derive(Debug, Parser)]
#[command(version, about = "BlueOS backbone recorder (MCAP)")]
pub struct RecorderCli {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(long, default_value = "/tmp")]
    pub recorder_path: String,

    #[arg(long)]
    pub schema_path: Option<String>,

    #[arg(long, value_name = "KEY=VALUE", num_args = 1..)]
    pub zkey: Vec<String>,

    #[arg(long, value_enum, default_value_t = McapCompression::Lz4)]
    pub mcap_compression: McapCompression,

    #[arg(long, default_value_t = DEFAULT_CHUNK_BYTES)]
    pub mcap_chunk_size: NonZeroU64,

    #[arg(long, default_value_t = DEFAULT_FLUSH_INTERVAL_SECS)]
    pub mcap_flush_interval_secs: NonZeroU64,
}

pub fn parse_cli(arguments: &[String]) -> RecorderCli {
    let expanded: Vec<String> = arguments
        .iter()
        .map(|argument| {
            shellexpand::env(argument)
                .inspect_err(|_error| warn!(argument, "Failed expanding arg, using literal"))
                .unwrap_or_else(|_| argument.clone().into())
                .into_owned()
        })
        .collect();
    RecorderCli::parse_from(expanded)
}

pub fn recorder_directory(cli: &RecorderCli) -> PathBuf {
    PathBuf::from(&cli.recorder_path)
}

pub fn schema_directory(cli: &RecorderCli) -> Option<PathBuf> {
    cli.schema_path.as_ref().map(PathBuf::from)
}

pub fn mcap_write_config(cli: &RecorderCli) -> McapWriteConfig {
    McapWriteConfig {
        compression: cli.mcap_compression,
        chunk_size: cli.mcap_chunk_size,
        flush_interval_secs: cli.mcap_flush_interval_secs,
    }
}

#[allow(dead_code)]
pub fn zenoh_key_overrides(cli: &RecorderCli) -> HashMap<String, String> {
    let mut configuration = HashMap::new();
    for entry in &cli.zkey {
        if let Some((key, value)) = entry.split_once('=') {
            configuration.insert(key.to_string(), value.to_string());
        } else {
            warn!(entry, "Invalid zkey format, expected KEY=VALUE");
        }
    }
    configuration
}
