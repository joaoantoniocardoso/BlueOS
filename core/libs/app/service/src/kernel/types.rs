//! Kernel types, constants, and rejection reasons.

use core::{error::Error, time::Duration};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use tokio::{
    sync::{mpsc, watch},
    task::JoinSet,
};

use blueos_comms::{CommsBackend, CommsError};
use blueos_domain::Domain;
use blueos_idl::Error as IdlError;
use blueos_jobs::{JobControl, JobId, Jobs, JobsError};

use crate::{
    builder::{Decode, EventEndpoint, JobOutput, JobsAccess, Refusal, StateEndpoint},
    clock::Clock,
    durable_state::DurableStateHandle,
    inbox::{Delivery, Input},
    logging::LogPublisherRuntime,
    metrics_registry::MetricsRegistry,
    projection::ProjectionRegistry,
    runtime_gauges::RuntimeGauges,
    settings::SettingsDriver,
    shutdown::IoInflight,
    tasks::TaskSupervisor,
};

use super::{io::IoExecutors, timers::TimerWheel};

/// How many Commands wait in the Inbox before a sender has to wait.
pub(crate) const INBOX_CAPACITY: usize = 256;
/// The encoding of the reason in a Query's error reply.
pub(crate) const REASON_ENCODING: &str = "text/plain";
/// The Job type every Service serves to replace its settings (D-11).
pub(crate) const UPDATE_SETTINGS: &str = "UpdateSettings";
/// The interface type of [`UPDATE_SETTINGS`], as `info` lists it.
pub(crate) const UPDATE_SETTINGS_ACTION: &str = "blueos_msgs/action/UpdateSettings";
/// The State every Service publishes its metrics on (D-12, D-35).
pub(crate) const METRICS: &str = "metrics";
/// The shortest time between two publications of the `metrics` State (D-35).
pub(crate) const METRICS_PERIOD: Duration = Duration::from_secs(1);
/// The line ROS 2 schema text puts before the schema of each message it depends on.
pub(crate) const SCHEMA_SEPARATOR: &str =
    "================================================================================";
/// The controls every Service serves on `command/<control>`. The answer of `AnswerPermission` comes from its body.
pub(crate) const CONTROLS: [JobControl; 4] = [
    JobControl::Cancel,
    JobControl::Pause,
    JobControl::Resume,
    JobControl::AnswerPermission { granted: false },
];

/// Turns the Job id and the body of a Command into what the Kernel applies.
pub(crate) type IntoInput<D> = Box<dyn Fn(JobId, Vec<u8>) -> Result<Input<D>, Rejection> + Send>;

/// One applied Command's Effects in application order.
#[cfg(feature = "testing")]
type EffectBatch<D> = Vec<
    blueos_domain::Effect<<D as Domain>::Tick, <D as Domain>::IoRequest, <D as Domain>::TimerKey>,
>;

/// Shared storage for recorded Effects when the harness asks not to run them.
#[cfg(feature = "testing")]
pub(crate) type EffectLogStorage<D> = Arc<std::sync::Mutex<Vec<EffectBatch<D>>>>;

/// Runs one Domain: the only writer of its Snapshot, and the owner of everything that can be stopped (the Inbox
/// and the endpoint adapters). Dropping it stops them.
pub struct Kernel<D: Domain, Context = ()> {
    pub(crate) service: &'static str,
    pub(crate) snapshot: D::Snapshot,
    pub(crate) inbox: mpsc::Receiver<Delivery<D>>,
    pub(crate) inbox_sender: Option<mpsc::Sender<Delivery<D>>>,
    pub(crate) states: Vec<PublishedState<D>>,
    pub(crate) settings: Option<SettingsEndpoint<D>>,
    pub(crate) durable: Option<DurableStateHandle<D>>,
    pub(crate) events: Vec<EventEndpoint<D>>,
    /// Decodes the Goal of each Job type, for a Job that executes once its permission is granted.
    pub(crate) goal_decoders: HashMap<String, Decode<D>>,
    /// Where the Jobs are when the Domain keeps them in its Snapshot.
    pub(crate) jobs_access: Option<JobsAccess<D>>,
    /// The Jobs, when the Domain does not keep them.
    pub(crate) own_jobs: Jobs,
    /// The standard `jobs` State, as the backbone last accepted it.
    pub(crate) jobs_latest: watch::Sender<Option<Bytes>>,
    /// The Feedback State, Job result Event and history of each Job type, in the order the Job types were added.
    pub(crate) job_outputs: Vec<JobTypeOutput<D>>,
    pub(crate) backend: Arc<dyn CommsBackend>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) timers: TimerWheel<D>,
    pub(crate) context: Arc<Context>,
    pub(crate) io: IoExecutors<D, Context>,
    pub(crate) snapshot_for_queries: Arc<tokio::sync::RwLock<D::Snapshot>>,
    #[cfg(feature = "testing")]
    pub(crate) effect_log: Option<EffectLogStorage<D>>,
    pub(crate) endpoints: JoinSet<()>,
    pub(crate) shutdown_request: tokio::sync::Mutex<Option<D::Request>>,
    pub(crate) shutdown_receiver: Option<watch::Receiver<bool>>,
    pub(crate) io_inflight: IoInflight,
    pub(crate) shutting_down: bool,
    pub(crate) tasks: TaskSupervisor,
    pub(crate) projections: ProjectionRegistry<D>,
    pub(crate) log_publisher: Option<LogPublisherRuntime>,
    /// Every metric the Service recorded (D-35).
    pub(crate) metrics: MetricsRegistry,
    /// The standard `metrics` State, as the backbone last accepted it.
    pub(crate) metrics_latest: watch::Sender<Option<Bytes>>,
    pub(crate) inbox_step_time: metrics::Histogram,
    pub(crate) inbox_depth: metrics::Gauge,
    pub(crate) runtime_gauges: RuntimeGauges,
}

/// The standard `settings` State and its persistence driver (D-11).
pub(crate) struct SettingsEndpoint<D: Domain> {
    pub(crate) key: String,
    pub(crate) encoding: String,
    pub(crate) driver: Arc<Mutex<Box<dyn SettingsDriver<D>>>>,
    pub(crate) latest: watch::Sender<Option<Bytes>>,
}

/// A State with its key and the last value the backbone accepted.
pub(crate) struct PublishedState<D: Domain> {
    pub(crate) key: String,
    pub(crate) endpoint: StateEndpoint<D>,
    /// Stored only after a publish succeeds, so a failed publish is retried on the next Command.
    pub(crate) latest: watch::Sender<Option<Bytes>>,
}

/// What one Job type publishes besides its status in the `jobs` State (D-12): its Feedback State on
/// `jobs/<JobType>/feedback`, its Job result Event on `jobs/<JobType>/result`, and its last finished Jobs, answered on
/// `jobs/<JobType>/history`.
pub(crate) struct JobTypeOutput<D: Domain> {
    pub(crate) job_type: String,
    pub(crate) output: JobOutput<D>,
    /// The Feedback State, as the backbone last accepted it.
    pub(crate) feedback_latest: watch::Sender<Option<Bytes>>,
    /// The answer of the history Query.
    pub(crate) history: watch::Sender<Option<Bytes>>,
}

/// One Job type's output after a step, encoded inside the step's transaction.
pub(crate) struct EncodedJobOutput {
    pub(crate) feedback: Result<Vec<u8>, IdlError>,
    pub(crate) history: Result<Vec<u8>, IdlError>,
    /// The Job result Event of each Job of the type that ended in the step.
    pub(crate) results: Vec<Result<Vec<u8>, IdlError>>,
}

/// The build declared Feedback or a Job result for `job_type`, but no Job type of that name.
#[derive(Debug, thiserror::Error)]
#[error("{job_type} declares Feedback or a Job result but is no Job type")]
pub(crate) struct OutputWithoutJobType {
    pub(crate) job_type: String,
}

/// Why the Kernel did not apply a Command. Its text is the reason in the rejected [`CommandAck`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum Rejection {
    /// The Domain rejected the Command, with its own reason.
    #[error("{0}")]
    Domain(Box<dyn Error + Send + Sync>),
    /// `handle` or a Projection panicked.
    #[error("the Command panicked, so nothing changed")]
    Panicked,
    /// A client Command arrived after shutdown started.
    #[error("the service is shutting down")]
    ShuttingDown,
    /// The body is not the Command endpoint's Message.
    #[error("the Request does not decode: {0}")]
    InvalidBody(IdlError),
    /// The endpoint's conversion refused the Message, with its own reason.
    #[error("{0}")]
    Refused(Refusal),
    /// The attachment of a Command is not the id of the Job it submits or controls.
    #[error("the Command's attachment is not a Job id")]
    NoJobId,
    /// The Service registered no settings, so there is nothing for `UpdateSettings` to change.
    #[error("the Service has no settings")]
    NoSettings,
    /// The Job a permission answer resumes has a type the Service does not register.
    #[error("there is no Job type {0}")]
    UnknownJobType(String),
    /// The Jobs refused the submit or the control.
    #[error(transparent)]
    Jobs(#[from] JobsError),
}

/// Why a Query or an IO query got no answer. Its text is the reason in the error reply.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Unanswered {
    /// The body is not the endpoint's request Message.
    #[error("the Query does not decode: {0}")]
    InvalidBody(IdlError),
    /// The endpoint's conversion or IO code refused the request, with its own reason.
    #[error("{0}")]
    Refused(Refusal),
    /// The Domain answered with a Response that the endpoint does not publish.
    #[error("the Domain's Response does not belong to this Query")]
    OtherResponse,
    /// The answer panicked.
    #[error("the Query panicked")]
    Panicked,
    /// The reply does not encode.
    #[error("the reply does not encode: {0}")]
    Encode(IdlError),
}

/// Why a State, an Event or an ack did not reach the backbone.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SendError {
    #[error("the Message does not encode: {0}")]
    Encode(#[from] IdlError),
    #[error(transparent)]
    Comms(#[from] CommsError),
}
