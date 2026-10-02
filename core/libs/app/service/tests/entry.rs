//! Multicall resolution and the common CLI flattened with each Service's arguments.

use core::convert::Infallible;

use std::{ffi::OsString, path::PathBuf, process::ExitCode};

use clap::Args;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_service::{
    Service, ServiceBuilder, ServiceContext, ServiceError,
    entry::{
        missing_feature, multicall_help, multicall_version, parse_service_cli, resolve, run, usage,
        verbosity_from_raw,
    },
};

#[derive(Args, Clone, Debug)]
struct FixtureArguments {
    #[arg(long, value_name = "FILE")]
    marker: Option<PathBuf>,
}

struct Fixture;

#[derive(Clone)]
struct FixtureSnapshot;

impl Service for FixtureService {
    type Domain = Fixture;
    type Context = ();
    type Arguments = FixtureArguments;

    const NAME: &'static str = "fixture";
    const VERSION: &'static str = "9.8.7";

    fn build(
        context: &ServiceContext<FixtureArguments>,
    ) -> Result<ServiceBuilder<Fixture>, ServiceError> {
        let _marker = context.arguments().marker.clone();
        Ok(ServiceBuilder::new(FixtureSnapshot))
    }
}

struct FixtureService;

impl Domain for Fixture {
    type Snapshot = FixtureSnapshot;
    type Request = Infallible;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut Self::Snapshot,
        _command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        _now: Now,
    ) -> Decision<Self> {
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

#[test]
fn common_flattened_with_service_flag_without_repeating_common_fields() {
    let parsed = parse_service_cli::<FixtureService>(
        [
            "fixture",
            "-v",
            "--settings-path",
            "/data/settings",
            "--zenoh-endpoint",
            "tcp/10.0.0.1:7447",
            "--marker",
            "/tmp/marker",
        ]
        .map(OsString::from),
    )
    .expect("parse");
    assert_eq!(parsed.common.verbose, 1);
    assert_eq!(
        parsed.common.settings_path.as_deref(),
        Some(PathBuf::from("/data/settings").as_path())
    );
    assert_eq!(parsed.common.zenoh_endpoint, "tcp/10.0.0.1:7447");
    assert_eq!(
        parsed.service.marker.as_deref(),
        Some(PathBuf::from("/tmp/marker").as_path())
    );
}

#[test]
fn paths_stay_literal_without_shell_expansion() {
    let parsed = parse_service_cli::<FixtureService>(
        [
            "fixture",
            "--settings-path",
            "$HOME/blueos",
            "--marker",
            "~/marker",
        ]
        .map(OsString::from),
    )
    .expect("parse");
    assert_eq!(
        parsed.common.settings_path.as_deref(),
        Some(PathBuf::from("$HOME/blueos").as_path())
    );
    assert_eq!(
        parsed.service.marker.as_deref(),
        Some(PathBuf::from("~/marker").as_path())
    );
}

#[test]
fn service_help_and_version_exit_success() {
    let help = parse_service_cli::<FixtureService>(["fixture", "--help"].map(OsString::from));
    assert!(help.is_err());
    assert_eq!(help.unwrap_err().exit_code(), ExitCode::SUCCESS);

    let version = parse_service_cli::<FixtureService>(["fixture", "--version"].map(OsString::from));
    assert!(version.is_err());
    assert_eq!(version.unwrap_err().exit_code(), ExitCode::SUCCESS);
}

#[test]
fn resolve_subcommand_style() {
    let arguments = ["blueos", "fixture", "--help"].map(OsString::from);
    let resolved = resolve(&arguments).expect("name");
    assert_eq!(resolved.0, "fixture");
    assert_eq!(
        resolved.1,
        [OsString::from("fixture"), OsString::from("--help")]
    );
}

#[test]
fn resolve_symlink_style() {
    let arguments = ["/usr/bin/fixture", "-h"].map(OsString::from);
    let resolved = resolve(&arguments).expect("name");
    assert_eq!(resolved.0, "fixture");
    assert_eq!(
        resolved.1,
        [OsString::from("fixture"), OsString::from("-h")]
    );
}

#[test]
fn verbosity_counted_from_raw_arguments() {
    let arguments = ["fixture", "-vv", "--marker", "x"].map(OsString::from);
    assert_eq!(verbosity_from_raw(&arguments), 2);
}

#[test]
fn run_builds_service_from_parsed_arguments() {
    let exit_code = run::<FixtureService>(
        ["fixture", "--marker", "/tmp/x"]
            .map(OsString::from)
            .to_vec(),
    );
    assert_eq!(exit_code, ExitCode::SUCCESS);
}

#[test]
fn multicall_help_and_version_exit_success() {
    assert_eq!(multicall_help(&["fixture"]), ExitCode::SUCCESS);
    assert_eq!(multicall_version("1.2.3"), ExitCode::SUCCESS);
}

#[test]
fn usage_lists_known_services() {
    assert_eq!(usage(&["fixture"]), ExitCode::from(2));
}

#[test]
fn missing_feature_message_names_service() {
    assert_eq!(missing_feature("fixture"), ExitCode::from(2));
}
