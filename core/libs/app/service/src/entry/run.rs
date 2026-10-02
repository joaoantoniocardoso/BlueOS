//! Start one Service from its command-line arguments.

use std::{ffi::OsString, process::ExitCode};

use tracing::error;
use tracing_subscriber::EnvFilter;

use crate::{Service, ServiceContext};

use super::{parse::parse_service_cli, verbosity::verbosity_from_raw};

/// Starts logging, parses the CLI, calls `S::build`, and returns an exit code.
///
/// ponytail: #47 will open the Session, run the Kernel, and map shutdown to exit codes. Today a successful `build`
/// exits 0 and a failed `build` exits 1.
pub fn run<S: Service>(arguments: Vec<OsString>) -> ExitCode {
    init_logging(verbosity_from_raw(&arguments));
    let parsed = match parse_service_cli::<S>(arguments) {
        Ok(parsed) => parsed,
        Err(error) => return error.exit_code(),
    };
    let _common = parsed.common;
    match S::build(&ServiceContext::new(parsed.service)) {
        Ok(_builder) => ExitCode::SUCCESS,
        Err(service_error) => {
            error!(%service_error, "The service could not start");
            ExitCode::from(1)
        }
    }
}

fn init_logging(verbosity: u8) {
    let default_level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
