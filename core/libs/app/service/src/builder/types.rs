//! Builder types and `ServiceBuilder::new`.

use core::{error::Error, future::Future, num::NonZeroU32, pin::Pin};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tokio::sync::watch;

use blueos_domain::{Command, Domain};
use blueos_idl::{Error as IdlError, msg::blueos_msgs::EndpointInfo};
use blueos_jobs::{JobId, JobNature, Jobs};

use crate::{
    durable_state::DurableStateRegistration,
    kernel::{Rejection, Unanswered, io::IoExecutors},
    metrics_registry::MetricsRegistry,
    settings::SettingsRegistration,
    tasks::TaskSpec,
};

/// One Command the Kernel delivers through the Inbox.
pub(crate) type InboxCommand<D> = Command<
    <D as Domain>::Request,
    <D as Domain>::IoResult,
    <D as Domain>::Tick,
    <D as Domain>::ObservedFact,
>;

/// Decodes the Goal of the Job a client submitted into the Domain's Request.
pub(crate) type Decode<D> =
    Arc<dyn Fn(JobId, &[u8]) -> Result<<D as Domain>::Request, Rejection> + Send + Sync>;

/// Computes and encodes a State from the Snapshot.
pub(crate) type Project<D> =
    Box<dyn Fn(&<D as Domain>::Snapshot) -> Result<Vec<u8>, IdlError> + Send + Sync>;

/// Turns a domain event into an encoded Event, or `None` when this Event endpoint does not publish it.
pub(crate) type Select<D> =
    Box<dyn Fn(&<D as Domain>::Event) -> Option<Result<Vec<u8>, IdlError>> + Send + Sync>;

/// Encodes the latest Feedback of an active Job from the Snapshot, or `None` while it has none.
pub(crate) type ProjectFeedback<D> =
    Box<dyn Fn(&<D as Domain>::Snapshot, JobId) -> Option<Result<Vec<u8>, IdlError>> + Send + Sync>;

/// Encodes the Job result of a Job from the Snapshot after the step that ended it.
pub(crate) type ProjectResult<D> =
    Box<dyn Fn(&<D as Domain>::Snapshot, JobId) -> Result<Vec<u8>, IdlError> + Send + Sync>;

/// Reads and changes the Jobs a Domain keeps in its Snapshot.
pub(crate) type JobsAccess<D> = (
    fn(&<D as Domain>::Snapshot) -> &Jobs,
    fn(&mut <D as Domain>::Snapshot) -> &mut Jobs,
);

/// Answers an IO query body with the encoded reply.
pub(crate) type Respond = Box<
    dyn Fn(Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Unanswered>> + Send>> + Send,
>;

/// Why an endpoint conversion or an IO query refused what a client sent. Its text is the reason the client gets.
pub type Refusal = Box<dyn Error + Send + Sync>;

/// Name, version and capabilities the Kernel publishes on the standard `info` query.
#[derive(Clone, Debug)]
pub(crate) struct ServiceMetadata {
    pub(crate) version: &'static str,
    pub(crate) build: &'static str,
    pub(crate) capabilities: &'static [&'static str],
}

/// Everything a Service declares in `build`: the initial Snapshot, then one call per endpoint. Each call converts
/// between a Message and the Domain's own types, so the Domain never sees a Message. `Context` is the type IO code
/// and Tasks receive; the Kernel gets the value itself when it starts.
#[must_use]
pub struct ServiceBuilder<D: Domain, Context = ()> {
    pub(crate) snapshot: D::Snapshot,
    pub(crate) metadata: ServiceMetadata,
    /// The folder the settings and the durable state live in, from the ServiceContext.
    pub(crate) settings_folder: Option<PathBuf>,
    pub(crate) manifest_endpoints: Vec<EndpointInfo>,
    pub(crate) io: IoExecutors<D, Context>,
    pub(crate) commands: Vec<CommandEndpoint<D>>,
    pub(crate) queries: Vec<(String, AnswerQuery<D>)>,
    pub(crate) io_queries: Vec<IoQueryEndpoint>,
    pub(crate) states: Vec<StateEndpoint<D>>,
    pub(crate) projections: Vec<crate::projection::RefreshProjection<D>>,
    pub(crate) events: Vec<EventEndpoint<D>>,
    pub(crate) settings: Option<SettingsRegistration<D>>,
    pub(crate) durable: Option<DurableStateDeclaration<D>>,
    /// Set when the Domain keeps the Jobs in its Snapshot; the Kernel keeps them otherwise.
    pub(crate) jobs: Option<JobsAccess<D>>,
    /// The Feedback and Job result each Job type declared, by Job type.
    pub(crate) job_outputs: HashMap<String, JobOutput<D>>,
    pub(crate) startup_commands: Vec<InboxCommand<D>>,
    pub(crate) tasks: Vec<TaskSpec<D, Context>>,
    pub(crate) shutdown_request: Option<D::Request>,
    pub(crate) shutdown_sender: Option<watch::Sender<bool>>,
    pub(crate) shutdown_receiver: Option<watch::Receiver<bool>>,
    /// The registry the Service's metrics are recorded in (D-35).
    pub(crate) metrics: MetricsRegistry,
}

/// The durable state a Service declared, which the Kernel opens in the Service's settings folder when it starts.
pub(crate) struct DurableStateDeclaration<D: Domain> {
    /// Opens the store of the Service named by the first argument in the settings folder of the second.
    pub(crate) open: fn(String, Option<PathBuf>, NonZeroU32) -> DurableStateRegistration<D>,
    pub(crate) version: NonZeroU32,
}

/// Answers one Query from the Snapshot and the request body.
pub(crate) type AnswerQuery<D> = Arc<
    dyn Fn(
            &<D as Domain>::Snapshot,
            &[u8],
            blueos_domain::Now,
        ) -> Result<(Vec<u8>, String), Unanswered>
        + Send
        + Sync,
>;

/// A Job type: a query on `blueos/v1/<service>/command/<name>` whose body is the Goal of a Job to submit.
pub(crate) struct CommandEndpoint<D: Domain> {
    pub(crate) name: String,
    pub(crate) nature: JobNature,
    pub(crate) decode: Decode<D>,
}

/// What a Job type publishes besides its status in the `jobs` State: its Feedback and its Job result (D-12).
pub(crate) struct JobOutput<D: Domain> {
    pub(crate) feedback: Option<ProjectFeedback<D>>,
    pub(crate) result: Option<ProjectResult<D>>,
    /// The message of the Feedback, as `info` describes it.
    pub(crate) feedback_type: MessageType,
    /// The message of the Job result, as `info` describes it.
    pub(crate) result_type: MessageType,
}

/// The schema name and text of a message, both empty for a Job type that declares no such message.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MessageType {
    pub(crate) name: &'static str,
    pub(crate) schema: &'static str,
}

/// An IO query endpoint: a query on `blueos/v1/<service>/query/<name>`, answered outside the Inbox.
pub(crate) struct IoQueryEndpoint {
    pub(crate) name: String,
    pub(crate) encoding: String,
    pub(crate) respond: Respond,
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

impl<D: Domain, Context> ServiceBuilder<D, Context> {
    /// A Service whose Domain starts from `snapshot`, with no endpoints yet.
    pub fn new(snapshot: D::Snapshot) -> Self {
        Self {
            snapshot,
            metadata: ServiceMetadata {
                version: "0.0.0",
                build: "dev",
                capabilities: &[],
            },
            settings_folder: None,
            manifest_endpoints: Vec::new(),
            io: IoExecutors {
                r#async: None,
                blocking: None,
            },
            commands: Vec::new(),
            queries: Vec::new(),
            io_queries: Vec::new(),
            states: Vec::new(),
            projections: Vec::new(),
            events: Vec::new(),
            settings: None,
            durable: None,
            jobs: None,
            job_outputs: HashMap::new(),
            startup_commands: Vec::new(),
            tasks: Vec::new(),
            shutdown_request: None,
            shutdown_sender: None,
            shutdown_receiver: None,
            metrics: MetricsRegistry::default(),
        }
    }
}

impl<D: Domain> Default for JobOutput<D> {
    fn default() -> Self {
        Self {
            feedback: None,
            result: None,
            feedback_type: MessageType::default(),
            result_type: MessageType::default(),
        }
    }
}
