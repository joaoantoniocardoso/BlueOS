use autopilot_preflight::Snapshot;
use blueos_configs::Configs;
use blueos_logging::error;
use clap::Parser;

#[derive(Parser)]
pub(crate) struct Cli {
    #[command(flatten)]
    pub common: blueos_cli::Common,
    #[arg(long)]
    pub moving: bool,
}

pub(crate) fn load_configs(cli: &Cli) -> Configs {
    match &cli.common.config {
        Some(path) => Configs::load(path).unwrap_or_else(|error| {
            error!("{error}");
            std::process::exit(1);
        }),
        None => Configs::default(),
    }
}

pub(crate) fn snapshot_from(cli: &Cli, configs: &Configs) -> Snapshot {
    let moving = cli.moving
        || configs
            .get("moving")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
    Snapshot {
        moving,
        ..Snapshot::default()
    }
}
