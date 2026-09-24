mod cli;
mod io;
mod local;
mod wire;

use std::ffi::OsString;

use blueos_cli::Argv;
use blueos_comms::{Endpoint, Session};
use blueos_cqrs::App;
use blueos_logging::error;
use blueos_service::{Adapters, Service};
use calibration_sensors::{
    AUTOPILOT_STATE_ACK, AUTOPILOT_STATE_OFFSETS, Calibration, Query, Snapshot,
};
use clap::Parser;

use cli::{Cli, from_cli, tokens};
use io::execute_io;
use local::run_local;
use wire::{parse_ack, parse_offsets};

pub fn run(args: impl IntoIterator<Item = OsString>) {
    let cli = Cli::parse_from(args);
    let _ = blueos_logging::init(cli.common.verbose); // already-initialized is success
    let configs = cli::load_configs(cli.common.config.as_deref());
    let args = tokens(cli.command);
    match Session::open(Endpoint::Local) {
        Ok(comms) => {
            let mut service: Service<Calibration> = Service::new(
                App::new(Snapshot::default()),
                Adapters {
                    comms,
                    configs,
                    cli: Argv::from_args(args.clone()),
                },
            );
            if args.first().map(String::as_str) == Some("snapshot") {
                let view = service.app.query(Query::GetSnapshot);
                println!(
                    "gyro={:?} baro={:?}",
                    view.snapshot.gyro, view.snapshot.baro
                );
                return;
            }
            service.on_io(execute_io);
            service.watch(AUTOPILOT_STATE_ACK, |sample| parse_ack(&sample.payload));
            service.watch(AUTOPILOT_STATE_OFFSETS, |sample| {
                parse_offsets(&sample.payload)
            });
            service.on_cli(from_cli);
            if let Err(error) = service.run() {
                error!("calibration: run failed: {error}");
            }
        }
        Err(error) => {
            error!("calibration: comms open failed: {error}");
            run_local(&args);
        }
    }
}
