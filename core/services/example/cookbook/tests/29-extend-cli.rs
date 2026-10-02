//! Extend the common CLI with service-specific `clap::Args`.

use core::convert::Infallible;
use std::{ffi::OsString, path::PathBuf};

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_service::{
    Service, ServiceBuilder, ServiceContext, ServiceError, entry::parse_service_cli,
};

struct CliCookbookService;

#[derive(Clone, Debug, clap::Args)]
struct CliCookbookArguments {
    #[arg(long, value_name = "FILE")]
    marker: Option<PathBuf>,
}

struct CliCookbook;

#[derive(Clone)]
struct CliCookbookSnapshot {
    #[expect(dead_code, reason = "carried from CLI for the build example")]
    marker: Option<PathBuf>,
}

impl Service for CliCookbookService {
    type Domain = CliCookbook;
    type Context = ();
    type Arguments = CliCookbookArguments;

    const NAME: &'static str = "cookbook_cli";
    const VERSION: &'static str = "1.0.0";

    fn build(
        context: &ServiceContext<CliCookbookArguments>,
    ) -> Result<ServiceBuilder<CliCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(CliCookbookSnapshot {
            marker: context.arguments().marker.clone(),
        }))
    }
}

impl Domain for CliCookbook {
    type Snapshot = CliCookbookSnapshot;
    type Request = Infallible;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut CliCookbookSnapshot,
        _command: Command<Infallible, Infallible, Infallible, Infallible>,
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
fn service_flags_flatten_with_the_common_cli() {
    let parsed = parse_service_cli::<CliCookbookService>(
        [
            "cookbook_cli",
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
    let _builder = CliCookbookService::build(&ServiceContext::new(
        parsed.service,
        blueos_service::testing::channel_session(),
    ))
    .expect("build");
}
