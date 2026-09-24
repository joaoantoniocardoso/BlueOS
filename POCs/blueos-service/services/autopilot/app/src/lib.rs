mod attach;
mod cli;

use std::ffi::OsString;

use autopilot_preflight::Autopilot;
use blueos_cli::Argv;
use blueos_comms::{Endpoint, Session};
use blueos_cqrs::App;
use blueos_logging::error;
use blueos_service::{Adapters, Service};
use clap::Parser;

pub fn run(args: impl IntoIterator<Item = OsString>) {
    let cli = cli::Cli::parse_from(args);
    let _ = blueos_logging::init(cli.common.verbose); // already-initialized is success
    let configs = cli::load_configs(&cli);
    let snapshot = cli::snapshot_from(&cli, &configs);
    let mut service: Service<Autopilot> = Service::new(
        App::new(snapshot),
        Adapters {
            comms: Session::open(Endpoint::Local).expect("session"),
            configs,
            cli: Argv::default(),
        },
    );
    attach::attach(&mut service);
    if let Err(error) = service.run() {
        error!("{error}");
        std::process::exit(1);
    }
}
