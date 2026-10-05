//! Generates each Service's endpoint registration from its endpoint manifest (D-26).
//!
//! A Service's app crate commits `endpoints.toml`, next to its `Cargo.toml`. It lists four kinds of endpoint, `job`,
//! `query`, `state` and `event`, each with its interface type in `type`. From it the generator writes two committed
//! files: `logic/api/src/endpoints.rs`, the `Conversions` trait its Domain implements, and `app/src/endpoints.rs`,
//! the Service `NAME`, the `Handlers` trait of the `custom` Job types and `io` Queries, and `register`.

use alloc::collections::BTreeMap;

use serde::Deserialize;
/// `endpoints.toml`. Each table maps an endpoint name to its interface type, in sorted order.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
    pub(crate) service: String,
    #[serde(default)]
    pub(crate) job: BTreeMap<String, JobEntry>,
    #[serde(default)]
    pub(crate) query: BTreeMap<String, QueryEntry>,
    #[serde(default)]
    pub(crate) state: BTreeMap<String, PublishedEntry>,
    #[serde(default)]
    pub(crate) event: BTreeMap<String, PublishedEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct JobEntry {
    #[serde(rename = "type")]
    pub(crate) interface_type: String,
    #[serde(default)]
    pub(crate) custom: bool,
    pub(crate) nature: Option<NatureEntry>,
}

/// What a Job type allows (D-36). A Job type without one is an instant Job type that allows nothing.
#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NatureEntry {
    #[serde(default)]
    pub(crate) lasting: bool,
    #[serde(default)]
    pub(crate) cancellable: bool,
    #[serde(default)]
    pub(crate) pausable: bool,
    #[serde(default)]
    pub(crate) needs_permission: bool,
}

/// A Query: `io = true` when an adapter answers it outside the Inbox, which implies its handler method.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct QueryEntry {
    #[serde(rename = "type")]
    pub(crate) interface_type: String,
    #[serde(default)]
    pub(crate) io: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PublishedEntry {
    #[serde(rename = "type")]
    pub(crate) interface_type: String,
}

/// One endpoint of the manifest, with its interface type.
pub(crate) struct Endpoint {
    pub(crate) kind: Kind,
    pub(crate) name: String,
    pub(crate) function: String,
    pub(crate) interface: InterfaceType,
    pub(crate) custom: bool,
    /// Set for a Job type that declares its nature, so its Goal mapping gets the Job id.
    pub(crate) nature: Option<NatureEntry>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Job,
    Query,
    IoQuery,
    State,
    Event,
}

/// An interface type, from its name `<package>/<form>/<Name>`, whose form is `action`, `srv` or `msg`.
pub(crate) struct InterfaceType {
    pub(crate) package: String,
    pub(crate) form: &'static str,
    pub(crate) name: String,
}

/// An endpoint the Kernel declares for a Service, at `blueos/v1/<service>/<key>`.
pub(crate) struct StandardEndpoint {
    pub(crate) name: &'static str,
    pub(crate) key: &'static str,
    pub(crate) interface_type: &'static str,
}

impl InterfaceType {
    /// The Rust path of one of its parts: `Goal`, `Result` or `Feedback` of a `.action`, `Request` or `Response`
    /// of a `.srv`, and `""` for a `.msg`.
    pub(crate) fn path(&self, part: &str) -> String {
        format!("{}::{}{part}", self.package, self.name)
    }

    pub(crate) fn schema_name(&self) -> String {
        format!("{}/{}/{}", self.package, self.form, self.name)
    }

    /// The schema name of one of its parts, as the TypeScript codec looks it up.
    pub(crate) fn part_schema_name(&self, part: &str) -> String {
        if part.is_empty() {
            self.schema_name()
        } else {
            format!("{}_{part}", self.schema_name())
        }
    }

    pub(crate) fn typescript_type(&self, part: &str) -> String {
        format!("{}{part}", self.name)
    }
}
