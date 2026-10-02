//! What a Service's `build` declares: the initial Snapshot and how the Domain meets the backbone.

use core::error::Error;
use std::path::PathBuf;

use blueos_api::{Message, cdr_encoding};
use blueos_domain::Domain;
use blueos_idl::{Error as IdlError, msg::blueos_msgs::SettingsEnvelope};
use blueos_settings::SettingsSchema;

use crate::settings::{SettingsRegistration, register_settings};

/// Decodes a Request body into the Domain's Request.
pub(crate) type Decode<D> = Box<dyn Fn(&[u8]) -> Result<<D as Domain>::Request, IdlError> + Send>;

/// Computes and encodes a State from the Snapshot.
pub(crate) type Project<D> =
    Box<dyn Fn(&<D as Domain>::Snapshot) -> Result<Vec<u8>, IdlError> + Send + Sync>;

/// Turns a domain event into an encoded Event, or `None` when this Event endpoint does not publish it.
pub(crate) type Select<D> =
    Box<dyn Fn(&<D as Domain>::Event) -> Option<Result<Vec<u8>, IdlError>> + Send + Sync>;

/// Everything a Service declares in `build`: the initial Snapshot, then one call per endpoint. Each call converts
/// between a Message and the Domain's own types, so the Domain never sees a Message.
#[must_use]
pub struct ServiceBuilder<D: Domain> {
    pub(crate) snapshot: D::Snapshot,
    pub(crate) commands: Vec<CommandEndpoint<D>>,
    pub(crate) states: Vec<StateEndpoint<D>>,
    pub(crate) events: Vec<EventEndpoint<D>>,
    pub(crate) settings: Option<SettingsRegistration<D>>,
}

/// A Command endpoint: a query on `blueos/v1/<service>/command/<name>` whose body is a Request.
pub(crate) struct CommandEndpoint<D: Domain> {
    pub(crate) name: String,
    pub(crate) decode: Decode<D>,
}

/// A State: published on `blueos/v1/<service>/state/<name>` when it changes, and readable there at any time.
pub(crate) struct StateEndpoint<D: Domain> {
    pub(crate) name: String,
    pub(crate) encoding: String,
    pub(crate) project: Project<D>,
}

/// An Event endpoint: published on `blueos/v1/<service>/event/<name>`, once per domain event it selects.
pub(crate) struct EventEndpoint<D: Domain> {
    pub(crate) name: String,
    pub(crate) encoding: String,
    pub(crate) select: Select<D>,
}

impl<D: Domain> ServiceBuilder<D> {
    /// A Service whose Domain starts from `snapshot`, with no endpoints yet.
    pub fn new(snapshot: D::Snapshot) -> Self {
        Self {
            snapshot,
            commands: Vec::new(),
            states: Vec::new(),
            events: Vec::new(),
            settings: None,
        }
    }

    /// Registers Python-compatible settings (D-11): the Kernel loads once at startup, owns `UpdateSettings`, and
    /// persists after each successful update.
    ///
    pub fn settings<S>(
        mut self,
        service_name: &str,
        config_folder: Option<PathBuf>,
        into_snapshot: impl Fn(&mut D::Snapshot, S) + Send + Sync + 'static,
        from_snapshot: impl Fn(&D::Snapshot) -> S + Send + Sync + 'static,
        into_request: impl Fn(SettingsEnvelope) -> Result<D::Request, Box<dyn Error + Send + Sync>>
        + Send
        + Sync
        + 'static,
    ) -> Self
    where
        S: SettingsSchema + Send + Sync + 'static,
    {
        self.settings = Some(register_settings(
            service_name.to_owned(),
            config_folder,
            into_snapshot,
            from_snapshot,
            into_request,
        ));
        self
    }

    /// Adds the Command endpoint `name`. Its body is an `M`, which `into_request` turns into the Domain's Request.
    /// A body that does not decode is rejected before it reaches the Inbox.
    pub fn command<M: Message + 'static>(
        mut self,
        name: &str,
        into_request: impl Fn(M) -> D::Request + Send + 'static,
    ) -> Self {
        self.commands.push(CommandEndpoint {
            name: name.to_owned(),
            decode: Box::new(move |body| M::decode(body).map(&into_request)),
        });
        self
    }

    /// Adds the State `name`, computed from the Snapshot by `projection` after every applied Command and published
    /// only when its encoded value changes. `projection` must be pure: it runs inside the Command's transaction, so
    /// a panic in it restores the Snapshot and rejects the Command.
    pub fn state<M: Message + 'static>(
        mut self,
        name: &str,
        projection: impl Fn(&D::Snapshot) -> M + Send + Sync + 'static,
    ) -> Self {
        self.states.push(StateEndpoint {
            name: name.to_owned(),
            encoding: cdr_encoding(M::SCHEMA_NAME),
            project: Box::new(move |snapshot| projection(snapshot).encode()),
        });
        self
    }

    /// Adds the Event endpoint `name`, which publishes the `M` that `select` returns for a domain event, after the
    /// Command that produced it is acknowledged. `select` returns `None` for the domain events this endpoint does
    /// not publish.
    pub fn event<M: Message + 'static>(
        mut self,
        name: &str,
        select: impl Fn(&D::Event) -> Option<M> + Send + Sync + 'static,
    ) -> Self {
        self.events.push(EventEndpoint {
            name: name.to_owned(),
            encoding: cdr_encoding(M::SCHEMA_NAME),
            select: Box::new(move |event| select(event).map(|message| message.encode())),
        });
        self
    }
}
