//! The trait every Service implements, and what its `context` and `build` receive and may return.

use core::error::Error;
use std::path::{Path, PathBuf};

use blueos_comms::CommsError;
use blueos_domain::Domain;

use crate::{builder::ServiceBuilder, command_sender::Session};

/// A BlueOS Service: one Domain, its name and version, its command-line arguments, a `context` that builds what IO
/// code and Tasks use, and a `build` that declares how the Domain meets the backbone. `endpoints` is generated from
/// the Service's endpoint manifest (D-26).
///
/// ```ignore
/// impl Service for Example {
///     type Domain = Pump;
///     type Context = ();
///     type Arguments = ExampleArguments;
///
///     const NAME: &'static str = endpoints::NAME;
///     const VERSION: &'static str = env!("CARGO_PKG_VERSION");
///
///     fn context(_service: &ServiceContext<ExampleArguments>) -> Result<(), ServiceError> {
///         Ok(())
///     }
///
///     fn build(
///         _service: &ServiceContext<ExampleArguments>,
///         _context: &(),
///     ) -> Result<ServiceBuilder<Pump>, ServiceError> {
///         Ok(endpoints::register(ServiceBuilder::new(PumpSnapshot::default())))
///     }
/// }
/// ```
pub trait Service {
    /// The pure logic the Kernel runs.
    type Domain: Domain;
    /// What IO code and Tasks receive by reference: every Port with its adapter, and plain tunables. It never holds
    /// a Projection; a Task captures the one it follows in `build`.
    type Context: Send + Sync + 'static;
    /// The service's own command-line arguments, added to the ones every Service has.
    type Arguments: clap::Args;

    /// The name in every key of the service (`blueos/v1/<NAME>/...`).
    const NAME: &'static str;
    /// The version reported to clients. Write `env!("CARGO_PKG_VERSION")` in the service crate, where it expands to
    /// the service's version.
    const VERSION: &'static str;
    /// The build label reported on `info`, for example a git revision or `dev`.
    const BUILD: &'static str = "dev";
    /// Capability strings reported on `info`, when the Service has any.
    const CAPABILITIES: &'static [&'static str] = &[];

    /// Builds the Context, the only way the shipped Service gets one. It may open what the arguments name, and it
    /// fills every Port with its real adapter. A test changes the result through `Harness::start_with` before
    /// `build` sees it.
    ///
    /// # Errors
    ///
    /// [`ServiceError::Build`] when what the arguments name cannot be opened.
    fn context(service: &ServiceContext<Self::Arguments>) -> Result<Self::Context, ServiceError>;

    /// Declares the initial Snapshot, every endpoint, Task and Projection. It is pure: no IO and no spawning, so a
    /// test that runs it exercises exactly the wiring that ships. The Kernel owns `context` once `build` returns.
    ///
    /// # Errors
    ///
    /// [`ServiceError::Build`] when the arguments cannot make a working Service, such as an argument out of range.
    fn build(
        service: &ServiceContext<Self::Arguments>,
        context: &Self::Context,
    ) -> Result<ServiceBuilder<Self::Domain, Self::Context>, ServiceError>;
}

/// What a Service's `context` and `build` may use: parsed CLI arguments, optional settings path, and the open
/// Session.
pub struct ServiceContext<Arguments> {
    arguments: Arguments,
    pub(crate) settings_path: Option<PathBuf>,
    session: Session,
}

/// Why a Service did not start.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// The service's `context` or `build` refused its arguments.
    #[error("the service could not be built")]
    Build(#[source] Box<dyn Error + Send + Sync>),
    /// The Zenoh session could not open.
    #[error("the Zenoh session could not open")]
    Session(#[source] CommsError),
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
    /// A Context holding the parsed command-line arguments and the Session opened before `build`.
    ///
    /// With the `testing` feature, settings and durable state use a fresh temporary directory so a test does not
    /// read or write the user config folder. Production startup uses [`Self::with_settings_path`]; an unset path
    /// there still resolves from `XDG_CONFIG_HOME` or `$HOME/.config`.
    pub fn new(arguments: Arguments, session: Session) -> Self {
        Self {
            arguments,
            settings_path: isolated_settings_path(),
            session,
        }
    }

    /// Parsed arguments, the optional `--settings-path` parent, and the Session.
    ///
    /// `None` is the production default and still resolves from `XDG_CONFIG_HOME` or `$HOME/.config`.
    pub fn with_settings_path(
        arguments: Arguments,
        settings_path: Option<PathBuf>,
        session: Session,
    ) -> Self {
        Self {
            arguments,
            settings_path,
            session,
        }
    }

    /// The Zenoh session opened before `build`.
    pub fn session(&self) -> &Session {
        &self.session
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

#[cfg(feature = "testing")]
fn isolated_settings_path() -> Option<PathBuf> {
    Some(isolated_test_config_parent())
}

#[cfg(not(feature = "testing"))]
fn isolated_settings_path() -> Option<PathBuf> {
    None
}

#[cfg(feature = "testing")]
fn isolated_test_config_parent() -> PathBuf {
    use core::sync::atomic::{AtomicU64, Ordering};

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "blueos-test-config-{}-{}",
        std::process::id(),
        sequence
    ));
    // ponytail: directories accumulate under the system temp folder until the OS cleans them, because the
    // Kernel writes settings after the ServiceContext is dropped. A guard owned by the Kernel would remove
    // them when the service stops.
    std::fs::create_dir_all(&directory).expect("the isolated test config directory");
    directory
}
