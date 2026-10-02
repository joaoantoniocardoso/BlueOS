//! The trait every Service implements, and what its `build` receives and may return.

use core::error::Error;
use std::path::{Path, PathBuf};

use blueos_comms::CommsError;
use blueos_domain::Domain;

use crate::builder::ServiceBuilder;

/// A BlueOS Service: one Domain, its name and version, its command-line arguments, and a `build` that declares how
/// the Domain meets the backbone.
///
/// ```ignore
/// impl Service for Example {
///     type Domain = Pump;
///     type Arguments = ExampleArguments;
///
///     const NAME: &'static str = "example";
///     const VERSION: &'static str = env!("CARGO_PKG_VERSION");
///
///     fn build(context: &ServiceContext<ExampleArguments>) -> Result<ServiceBuilder<Pump>, ServiceError> {
///         Ok(ServiceBuilder::new(PumpSnapshot::default())
///             .command("SetLevel", |request: SetLevelRequest| PumpRequest::SetLevel(request.level))
///             .state("pump", PumpState::from))
///     }
/// }
/// ```
pub trait Service {
    /// The pure logic the Kernel runs.
    type Domain: Domain;
    /// The service's own command-line arguments, added to the ones every Service has.
    type Arguments: clap::Args;

    /// The name in every key of the service (`blueos/v1/<NAME>/...`).
    const NAME: &'static str;
    /// The version reported to clients. Write `env!("CARGO_PKG_VERSION")` in the service crate, where it expands to
    /// the service's version.
    const VERSION: &'static str;

    /// Declares the initial Snapshot and every endpoint. It is pure: no IO and no spawning, so a test that calls it
    /// exercises exactly the wiring that ships.
    ///
    /// # Errors
    ///
    /// [`ServiceError::Build`] when the Context cannot make a working Service, such as an argument out of range.
    fn build(
        context: &ServiceContext<Self::Arguments>,
    ) -> Result<ServiceBuilder<Self::Domain>, ServiceError>;
}

/// What a Service's `build` and IO code may use: its command-line arguments.
pub struct ServiceContext<Arguments> {
    arguments: Arguments,
    settings_path: Option<PathBuf>,
}

/// Why a Service did not start.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// The service's `build` refused its Context.
    #[error("the service could not be built")]
    Build(#[source] Box<dyn Error + Send + Sync>),
    /// The backbone refused an endpoint. Startup stops, so a Service never runs with an endpoint missing.
    #[error("the endpoint {key:?} could not be declared")]
    DeclareEndpoint {
        /// The endpoint's key.
        key: String,
        /// Why the backbone refused it.
        #[source]
        source: CommsError,
    },
    /// Settings could not be loaded or persisted.
    #[error(transparent)]
    Settings(#[from] blueos_settings::SettingsError),
}

impl<Arguments> ServiceContext<Arguments> {
    /// A Context holding the parsed command-line arguments.
    pub fn new(arguments: Arguments) -> Self {
        Self {
            arguments,
            settings_path: None,
        }
    }

    /// Like [`Self::new`], with the optional `--settings-path` parent directory.
    pub fn with_settings_path(arguments: Arguments, settings_path: Option<PathBuf>) -> Self {
        Self {
            arguments,
            settings_path,
        }
    }

    /// The service's own command-line arguments.
    pub fn arguments(&self) -> &Arguments {
        &self.arguments
    }

    /// Parent directory for this service's settings folder, when the entry layer set one.
    pub fn settings_path(&self) -> Option<&Path> {
        self.settings_path.as_deref()
    }
}
