//! Minimal Service used to exercise the multicall binary in tests and with `--features probe`.

use core::convert::Infallible;

use clap::Args;

use blueos_domain::{Command, Decision, Domain, Outcome};

use crate::{Service, ServiceBuilder, ServiceContext, ServiceError};

/// A no-op Service for CLI and multicall tests.
pub struct ProbeService;

/// Extra arguments for [`ProbeService`].
#[derive(Args, Clone, Debug)]
pub struct ProbeArguments {
    /// A path passed through unchanged (no shell expansion).
    #[arg(long, value_name = "FILE")]
    pub marker: Option<std::path::PathBuf>,
}

/// Domain for [`ProbeService`].
pub struct Probe;

/// Snapshot for [`Probe`].
#[derive(Clone)]
pub struct ProbeSnapshot;

impl Service for ProbeService {
    type Domain = Probe;
    type Arguments = ProbeArguments;

    const NAME: &'static str = "probe";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn build(
        context: &ServiceContext<ProbeArguments>,
    ) -> Result<ServiceBuilder<Probe>, ServiceError> {
        let _marker = context.arguments().marker.clone();
        Ok(ServiceBuilder::new(ProbeSnapshot))
    }
}

impl Domain for Probe {
    type Snapshot = ProbeSnapshot;
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
        _now: blueos_domain::Now,
    ) -> Decision<Self> {
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }
}
