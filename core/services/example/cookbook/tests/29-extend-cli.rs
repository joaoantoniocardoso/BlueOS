//! Question 29: how do I extend the CLI with a service flag?
//!
//! The answer is `type Arguments` on the `Service`: a `clap::Args` struct the Kernel flattens into the common CLI
//! (D-25). `build` and `context` read it through `ServiceContext::arguments`.

use core::convert::Infallible;
use std::{ffi::OsString, path::PathBuf};

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::LevelResponse;
use blueos_service::{
    Service, ServiceBuilder, ServiceContext, ServiceError, entry::parse_service_cli,
    testing::Harness,
};

struct CliCookbookService;

// Only service-specific flags go here; `-v`, `--settings-path` and the Zenoh flags are common and parsed by the Kernel.
#[derive(Clone, Debug, clap::Args)]
struct CliCookbookArguments {
    #[arg(long, value_name = "FILE")]
    marker: Option<PathBuf>,
}

struct CliCookbook;

#[derive(Clone)]
struct CliCookbookSnapshot {
    marker: Option<PathBuf>,
}

impl Service for CliCookbookService {
    type Domain = CliCookbook;
    type Context = ();
    type Arguments = CliCookbookArguments;

    const NAME: &'static str = "cookbook_cli";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<CliCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<CliCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<CliCookbook>, ServiceError> {
        // A flag reaches the Domain by being copied into the initial Snapshot; the Domain never sees the CLI.
        Ok(ServiceBuilder::new(CliCookbookSnapshot {
            marker: service.arguments().marker.clone(),
        })
        .state("cli", |snapshot: &CliCookbookSnapshot| LevelResponse {
            level: u8::from(snapshot.marker.is_some()),
            max_level: snapshot
                .marker
                .as_ref()
                .map(|path| path.as_os_str().len().min(255) as u8)
                .unwrap_or(0),
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

// `parse_service_cli` is the production parser, so this proves both flag sets parse together.
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
    let _builder = CliCookbookService::build(
        &ServiceContext::new(parsed.service, blueos_service::testing::channel_session()),
        &(),
    )
    .expect("build");
}

// The harness takes the Arguments value directly, so a test sets the flag without argv.
#[tokio::test(start_paused = true)]
async fn service_specific_flag_reaches_the_domain_snapshot() {
    let marker = PathBuf::from("/tmp/marker");
    let harness = Harness::<CliCookbookService>::start(CliCookbookArguments {
        marker: Some(marker.clone()),
    })
    .await
    .expect("start");
    let published = harness.state::<LevelResponse>("cli").await.unwrap();
    assert_eq!(published.level, 1);
    assert_eq!(published.max_level, marker.as_os_str().len() as u8);
}
