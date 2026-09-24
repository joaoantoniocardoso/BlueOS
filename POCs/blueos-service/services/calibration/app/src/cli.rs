use std::path::Path;

use blueos_configs::Configs;
use blueos_logging::error;
use calibration_sensors::Command;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub(crate) struct Cli {
    #[command(flatten)]
    pub common: blueos_cli::Common,
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Subcommand)]
pub(crate) enum CliCommand {
    Gyro,
    Baro,
    Stationary,
    Cancel,
    Snapshot,
}

pub(crate) fn load_configs(path: Option<&Path>) -> Configs {
    match path {
        Some(path) => Configs::load(path).unwrap_or_else(|error| {
            error!("{error}");
            std::process::exit(1);
        }),
        None => Configs::default(),
    }
}

pub(crate) fn tokens(command: Option<CliCommand>) -> Vec<String> {
    match command {
        Some(CliCommand::Gyro) => vec!["gyro".into()],
        Some(CliCommand::Baro) => vec!["baro".into()],
        Some(CliCommand::Stationary) => vec!["stationary".into()],
        Some(CliCommand::Cancel) => vec!["cancel".into()],
        Some(CliCommand::Snapshot) => vec!["snapshot".into()],
        None => Vec::new(),
    }
}

pub(crate) fn from_cli(args: &[String]) -> Option<Command> {
    match args.first().map(String::as_str) {
        Some("gyro") => Some(Command::StartGyro),
        Some("baro") => Some(Command::StartBaro),
        Some("stationary") => Some(Command::StartStationary),
        Some("cancel") => Some(Command::Cancel),
        _ => None,
    }
}
