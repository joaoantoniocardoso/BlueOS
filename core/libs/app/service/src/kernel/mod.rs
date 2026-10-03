//! The Kernel: the Inbox loop that applies every Command to the Domain and publishes what changed.

mod effects;
pub(crate) mod io;
mod timers;

use core::{error::Error, panic::AssertUnwindSafe, time::Duration};
use core::{
    future::pending,
    sync::atomic::{AtomicBool, Ordering},
};
use std::{
    collections::HashMap,
    panic,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use futures_util::FutureExt;
use tokio::{
    sync::{mpsc, watch},
    task::JoinSet,
    time::{Instant, MissedTickBehavior},
};
use tracing::warn;

use blueos_api::{
    CommandAck, Message, cdr_encoding, command_key, event_key, info_query_key, job_feedback_key,
    job_history_key, job_result_key, jobs_key, query_key, service_liveliness_key, settings_key,
    state_key, status_state_key,
};
use blueos_comms::{CommsBackend, CommsError, Query, QueryBody, Queryable, Sample};
use blueos_domain::{Command, Domain, Outcome};
use blueos_idl::{
    Error as IdlError,
    msg::blueos_msgs::{
        EndpointInfo, JobFeedback, JobFeedbackList, JobList, JobResult, PermissionAnswer,
        ServiceInfo, ServiceMetrics, ServiceStatus, ServiceStatusStatus, UpdateSettingsFeedback,
        UpdateSettingsGoal, UpdateSettingsResult,
    },
};
use blueos_jobs::{JobControl, JobEnd, JobId, JobNature, JobStatus, Jobs, JobsError, Submitted};

use crate::{
    builder::{
        AnswerQuery, Decode, EventEndpoint, InboxCommand, JobOutput, JobsAccess, MessageType,
        Refusal, Respond, ServiceBuilder, StateEndpoint, job_list, job_status,
    },
    clock::Clock,
    command_sender::{CommandSender, Session, command_ack},
    durable_state::{DurablePersister, DurableStateHandle},
    inbox::{CommandReply, Delivery, Input},
    inbox_recovery::{self, log_caught_panic},
    logging::LogPublisherRuntime,
    metrics_registry::MetricsRegistry,
    projection::ProjectionRegistry,
    run_outcome::RunOutcome,
    runtime_gauges::RuntimeGauges,
    service::ServiceError,
    settings::{SettingsDriver, settings_encoding},
    shutdown::{IoInflight, SHUTDOWN_IO_DRAIN_TIMEOUT, wait_for_shutdown_signal},
    sync::lock_unpoisoned,
    tasks::{TaskSupervisor, hold_liveliness_until_cancelled},
};

use effects::{apply_sync_effects, io_requests};
use io::{IoExecutors, spawn_io_chain};
use timers::TimerWheel;

/// How many Commands wait in the Inbox before a sender has to wait.
const INBOX_CAPACITY: usize = 256;
/// The encoding of the reason in a Query's error reply.
const REASON_ENCODING: &str = "text/plain";
/// The Job type every Service serves to replace its settings (D-11).
const UPDATE_SETTINGS: &str = "UpdateSettings";
/// The interface type of [`UPDATE_SETTINGS`], as `info` lists it.
const UPDATE_SETTINGS_ACTION: &str = "blueos_msgs/action/UpdateSettings";
/// The State every Service publishes its metrics on (D-12, D-35).
const METRICS: &str = "metrics";
/// The shortest time between two publications of the `metrics` State (D-35).
const METRICS_PERIOD: Duration = Duration::from_secs(1);
/// The histogram of how long the Inbox took to apply each Command, in seconds.
const INBOX_STEP_TIME: &str = "inbox_step_seconds";
/// The gauge of how many Commands waited in the Inbox when the last step began.
const INBOX_DEPTH: &str = "inbox_depth";
/// The line ROS 2 schema text puts before the schema of each message it depends on.
const SCHEMA_SEPARATOR: &str =
    "================================================================================";
/// The controls every Service serves on `command/<control>`. The answer of `AnswerPermission` comes from its body.
const CONTROLS: [JobControl; 4] = [
    JobControl::Cancel,
    JobControl::Pause,
    JobControl::Resume,
    JobControl::AnswerPermission { granted: false },
];

/// Turns the Job id and the body of a Command into what the Kernel applies.
type IntoInput<D> = Box<dyn Fn(JobId, Vec<u8>) -> Result<Input<D>, Rejection> + Send>;

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
    service: &'static str,
    snapshot: D::Snapshot,
    inbox: mpsc::Receiver<Delivery<D>>,
    inbox_sender: Option<mpsc::Sender<Delivery<D>>>,
    states: Vec<PublishedState<D>>,
    settings: Option<SettingsEndpoint<D>>,
    durable: Option<DurableStateHandle<D>>,
    events: Vec<EventEndpoint<D>>,
    /// Decodes the Goal of each Job type, for a Job that executes once its permission is granted.
    goal_decoders: HashMap<String, Decode<D>>,
    /// Where the Jobs are when the Domain keeps them in its Snapshot.
    jobs_access: Option<JobsAccess<D>>,
    /// The Jobs, when the Domain does not keep them.
    own_jobs: Jobs,
    /// The standard `jobs` State, as the backbone last accepted it.
    jobs_latest: watch::Sender<Option<Bytes>>,
    /// The Feedback State, Job result Event and history of each Job type, in the order the Job types were added.
    job_outputs: Vec<JobTypeOutput<D>>,
    backend: Arc<dyn CommsBackend>,
    clock: Arc<dyn Clock>,
    timers: TimerWheel<D>,
    context: Arc<Context>,
    io: IoExecutors<D, Context>,
    snapshot_for_queries: Arc<tokio::sync::RwLock<D::Snapshot>>,
    #[cfg(feature = "testing")]
    effect_log: Option<EffectLogStorage<D>>,
    endpoints: JoinSet<()>,
    shutdown_request: tokio::sync::Mutex<Option<D::Request>>,
    shutdown_receiver: Option<watch::Receiver<bool>>,
    io_inflight: IoInflight,
    shutting_down: bool,
    tasks: TaskSupervisor,
    projections: ProjectionRegistry<D>,
    log_publisher: Option<LogPublisherRuntime>,
    /// Every metric the Service recorded (D-35).
    metrics: MetricsRegistry,
    /// The standard `metrics` State, as the backbone last accepted it.
    metrics_latest: watch::Sender<Option<Bytes>>,
    inbox_step_time: metrics::Histogram,
    inbox_depth: metrics::Gauge,
    runtime_gauges: RuntimeGauges,
}

/// The standard `settings` State and its persistence driver (D-11).
struct SettingsEndpoint<D: Domain> {
    key: String,
    encoding: String,
    driver: Arc<Mutex<Box<dyn SettingsDriver<D>>>>,
    latest: watch::Sender<Option<Bytes>>,
}

/// A State with its key and the last value the backbone accepted.
struct PublishedState<D: Domain> {
    key: String,
    endpoint: StateEndpoint<D>,
    /// Stored only after a publish succeeds, so a failed publish is retried on the next Command.
    latest: watch::Sender<Option<Bytes>>,
}

/// What one Job type publishes besides its status in the `jobs` State (D-12): its Feedback State on
/// `jobs/<JobType>/feedback`, its Job result Event on `jobs/<JobType>/result`, and its last finished Jobs, answered on
/// `jobs/<JobType>/history`.
struct JobTypeOutput<D: Domain> {
    job_type: String,
    output: JobOutput<D>,
    /// The Feedback State, as the backbone last accepted it.
    feedback_latest: watch::Sender<Option<Bytes>>,
    /// The answer of the history Query.
    history: watch::Sender<Option<Bytes>>,
}

/// One Job type's output after a step, encoded inside the step's transaction.
struct EncodedJobOutput {
    feedback: Result<Vec<u8>, IdlError>,
    history: Result<Vec<u8>, IdlError>,
    /// The Job result Event of each Job of the type that ended in the step.
    results: Vec<Result<Vec<u8>, IdlError>>,
}

/// The build declared Feedback or a Job result for `job_type`, but no Job type of that name.
#[derive(Debug, thiserror::Error)]
#[error("{job_type} declares Feedback or a Job result but is no Job type")]
struct OutputWithoutJobType {
    job_type: String,
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
enum SendError {
    #[error("the Message does not encode: {0}")]
    Encode(#[from] IdlError),
    #[error(transparent)]
    Comms(#[from] CommsError),
}

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    /// Declares every endpoint of `builder` on `backend` and publishes the initial States, so every endpoint answers
    /// once this returns. The Kernel owns `context` from here on and hands it to IO code and Tasks.
    ///
    /// # Errors
    ///
    /// [`ServiceError::DeclareEndpoint`] when the backbone refuses an endpoint.
    pub async fn start(
        service: &'static str,
        builder: ServiceBuilder<D, Context>,
        context: Context,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, ServiceError> {
        #[cfg(feature = "testing")]
        {
            Self::boot(service, builder, context, backend, clock, None).await
        }
        #[cfg(not(feature = "testing"))]
        {
            Self::boot(service, builder, context, backend, clock).await
        }
    }

    /// Like [`Self::start`], optionally recording Effects without running IO or timers (harness only).
    #[cfg(feature = "testing")]
    pub async fn start_with_effect_log(
        service: &'static str,
        builder: ServiceBuilder<D, Context>,
        context: Context,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
        effect_log: Option<EffectLogStorage<D>>,
    ) -> Result<Self, ServiceError> {
        Self::boot(service, builder, context, backend, clock, effect_log).await
    }

    /// Runs the service log publisher flush as the last shutdown step (D-04, D-13).
    pub fn attach_log_publisher(&mut self, runtime: LogPublisherRuntime) {
        self.log_publisher = Some(runtime);
    }

    /// Handle for waiting on debounced durable writes in tests.
    #[cfg(feature = "testing")]
    pub fn durable_write_flush(&self) -> Option<crate::durable_state::DurableWriteFlush> {
        self.durable
            .as_ref()
            .map(|durable| durable.persister.flush_handle())
    }

    async fn boot(
        service: &'static str,
        builder: ServiceBuilder<D, Context>,
        context: Context,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
        #[cfg(feature = "testing")] effect_log: Option<EffectLogStorage<D>>,
    ) -> Result<Self, ServiceError> {
        let mut builder = builder
            .job_feedback(UPDATE_SETTINGS, |_, _| None::<UpdateSettingsFeedback>)
            .job_result(UPDATE_SETTINGS, |_, _| UpdateSettingsResult::default());
        let mut startup_commands = builder.startup_commands;
        let shutdown_request = tokio::sync::Mutex::new(builder.shutdown_request);
        let durable_registration = builder.durable.take().map(|declaration| {
            (declaration.open)(
                service.to_owned(),
                builder.settings_folder.clone(),
                declaration.version,
            )
        });
        if let Some(registration) = &durable_registration {
            registration
                .store
                .ensure_directory()
                .map_err(ServiceError::Settings)?;
            if (registration.restore_from_disk)(&mut builder.snapshot, clock.as_ref()) {
                startup_commands.push(Command::Tick(registration.restored_tick.clone()));
            }
        }
        let shutdown_receiver = builder.shutdown_receiver;
        let (inbox_sender, inbox) = mpsc::channel(INBOX_CAPACITY);
        let snapshot_for_queries = Arc::new(tokio::sync::RwLock::new(builder.snapshot.clone()));
        let job_type_names: Vec<String> = builder
            .commands
            .iter()
            .map(|command| command.name.clone())
            .chain([UPDATE_SETTINGS.to_owned()])
            .collect();
        let service_info = ServiceInfo {
            name: service.to_owned(),
            version: builder.metadata.version.to_owned(),
            build: builder.metadata.build.to_owned(),
            capabilities: builder
                .metadata
                .capabilities
                .iter()
                .map(|capability| (*capability).to_owned())
                .collect(),
            endpoints: builder
                .manifest_endpoints
                .iter()
                .cloned()
                .chain([
                    EndpointInfo {
                        kind: "job".to_owned(),
                        name: UPDATE_SETTINGS.to_owned(),
                        key: command_key(service, UPDATE_SETTINGS),
                        interface_type: UPDATE_SETTINGS_ACTION.to_owned(),
                        schema: blueos_idl::schema(UPDATE_SETTINGS_ACTION)
                            .unwrap_or_default()
                            .to_owned(),
                    },
                    EndpointInfo {
                        kind: "state".to_owned(),
                        name: "jobs".to_owned(),
                        key: jobs_key(service),
                        interface_type: JobList::SCHEMA_NAME.to_owned(),
                        schema: JobList::SCHEMA.to_owned(),
                    },
                    EndpointInfo {
                        kind: "state".to_owned(),
                        name: METRICS.to_owned(),
                        key: state_key(service, METRICS),
                        interface_type: ServiceMetrics::SCHEMA_NAME.to_owned(),
                        schema: ServiceMetrics::SCHEMA.to_owned(),
                    },
                ])
                .chain(job_type_names.iter().flat_map(|job_type| {
                    let (feedback_type, result_type) = builder
                        .job_outputs
                        .get(job_type)
                        .map(|output| (output.feedback_type, output.result_type))
                        .unwrap_or_default();
                    [
                        EndpointInfo {
                            kind: "state".to_owned(),
                            name: format!("jobs/{job_type}/feedback"),
                            key: job_feedback_key(service, job_type),
                            interface_type: JobFeedbackList::SCHEMA_NAME.to_owned(),
                            schema: carrying::<JobFeedbackList>(feedback_type),
                        },
                        EndpointInfo {
                            kind: "event".to_owned(),
                            name: format!("jobs/{job_type}/result"),
                            key: job_result_key(service, job_type),
                            interface_type: JobResult::SCHEMA_NAME.to_owned(),
                            schema: carrying::<JobResult>(result_type),
                        },
                        EndpointInfo {
                            kind: "query".to_owned(),
                            name: format!("jobs/{job_type}/history"),
                            key: job_history_key(service, job_type),
                            interface_type: JobList::SCHEMA_NAME.to_owned(),
                            schema: JobList::SCHEMA.to_owned(),
                        },
                    ]
                }))
                .collect(),
        };
        let info_key = info_query_key(service);
        let info_encoding = cdr_encoding(ServiceInfo::SCHEMA_NAME);
        let info_payload = Bytes::from(
            service_info
                .encode()
                .map_err(|error| ServiceError::Build(Box::new(error)))?,
        );
        let info_queryable = declare(&*backend, info_key.clone()).await?;
        let status_key = status_state_key(service);
        let status_encoding = cdr_encoding(ServiceStatus::SCHEMA_NAME);
        let status_latest = watch::Sender::new(None);
        let status_queryable = declare(&*backend, status_key.clone()).await?;
        let metrics_queryable = declare(&*backend, state_key(service, METRICS)).await?;
        let (inbox_step_time, inbox_depth) = metrics::with_local_recorder(&builder.metrics, || {
            (
                metrics::histogram!(INBOX_STEP_TIME),
                metrics::gauge!(INBOX_DEPTH),
            )
        });
        let runtime_gauges = RuntimeGauges::new(&builder.metrics);
        let task_supervisor = TaskSupervisor::new(
            service,
            Arc::clone(&backend),
            status_key.clone(),
            status_latest.clone(),
            builder.metrics.clone(),
        );
        let task_specs = builder.tasks;
        let update_settings_queryable =
            declare(&*backend, command_key(service, UPDATE_SETTINGS)).await?;
        let mut pending_settings_serve = None;
        let mut settings = None;
        if let Some(registration) = builder.settings {
            let mut driver =
                (registration.start)(service.to_owned(), builder.settings_folder.clone())?;
            driver.load_into(&mut builder.snapshot)?;
            let driver = Arc::new(Mutex::new(driver));
            let key = settings_key(service);
            let queryable = declare(&*backend, key.clone()).await?;
            let encoding = settings_encoding();
            let latest = watch::Sender::new(None);
            pending_settings_serve =
                Some((queryable, key.clone(), encoding.clone(), latest.subscribe()));
            settings = Some(SettingsEndpoint {
                key,
                encoding,
                driver,
                latest,
            });
        }
        let mut pending_commands = Vec::new();
        let mut goal_decoders = HashMap::new();
        let mut job_outputs = Vec::new();
        let mut pending_job_outputs = Vec::new();
        for command in builder.commands {
            let queryable = declare(&*backend, command_key(service, &command.name)).await?;
            let into_input: IntoInput<D> = {
                let decode = Arc::clone(&command.decode);
                let job_type = command.name.clone();
                let nature = command.nature;
                Box::new(move |job_id, goal| {
                    let request = decode(job_id, &goal)?;
                    Ok(Input::Submit {
                        job_id,
                        job_type: job_type.clone(),
                        goal,
                        nature,
                        request,
                    })
                })
            };
            pending_commands.push((queryable, into_input));
            goal_decoders.insert(command.name, command.decode);
        }
        for job_type in job_type_names {
            let feedback_key = job_feedback_key(service, &job_type);
            let history_key = job_history_key(service, &job_type);
            let job_output = JobTypeOutput {
                output: builder.job_outputs.remove(&job_type).unwrap_or_default(),
                job_type,
                feedback_latest: watch::Sender::new(None),
                history: watch::Sender::new(None),
            };
            pending_job_outputs.push((
                declare(&*backend, feedback_key.clone()).await?,
                feedback_key,
                job_output.feedback_latest.subscribe(),
                declare(&*backend, history_key.clone()).await?,
                history_key,
                job_output.history.subscribe(),
            ));
            job_outputs.push(job_output);
        }
        if let Some(job_type) = builder.job_outputs.keys().next() {
            return Err(ServiceError::Build(Box::new(OutputWithoutJobType {
                job_type: job_type.clone(),
            })));
        }
        for control in CONTROLS {
            let queryable = declare(&*backend, command_key(service, &control.to_string())).await?;
            let into_input: IntoInput<D> = Box::new(move |job_id, body| {
                let control = match control {
                    JobControl::AnswerPermission { .. } => JobControl::AnswerPermission {
                        granted: PermissionAnswer::decode(&body)
                            .map_err(Rejection::InvalidBody)?
                            .granted,
                    },
                    JobControl::Cancel | JobControl::Pause | JobControl::Resume => control,
                };
                Ok(Input::Control { job_id, control })
            });
            pending_commands.push((queryable, into_input));
        }
        let jobs_queryable = declare(&*backend, jobs_key(service)).await?;
        let jobs_latest = watch::Sender::new(None);
        let state_endpoints = builder
            .states
            .into_iter()
            .map(|endpoint| (state_key(service, &endpoint.name), endpoint));
        let mut states = Vec::new();
        let mut pending_states = Vec::new();
        for (key, endpoint) in state_endpoints {
            let queryable = declare(&*backend, key.clone()).await?;
            let latest = watch::Sender::new(None);
            pending_states.push((
                queryable,
                key.clone(),
                endpoint.encoding.clone(),
                latest.subscribe(),
            ));
            states.push(PublishedState {
                key,
                endpoint,
                latest,
            });
        }
        let mut pending_queries = Vec::new();
        for (name, answer) in builder.queries {
            let queryable = declare(&*backend, query_key(service, &name)).await?;
            pending_queries.push((queryable, answer));
        }
        let mut pending_io_queries = Vec::new();
        for endpoint in builder.io_queries {
            let queryable = declare(&*backend, query_key(service, &endpoint.name)).await?;
            pending_io_queries.push((queryable, endpoint.respond, endpoint.encoding));
        }
        let projections = ProjectionRegistry::new(builder.projections);
        let durable = durable_registration.map(|registration| DurableStateHandle {
            persister: DurablePersister::spawn(Arc::clone(&clock), registration.store),
            serialize: registration.serialize,
            changed: registration.changed,
        });
        let session: Session = Arc::clone(&backend);
        let mut kernel = Self {
            service,
            snapshot: builder.snapshot,
            inbox,
            inbox_sender: Some(inbox_sender),
            states,
            settings,
            durable,
            events: builder.events,
            goal_decoders,
            jobs_access: builder.jobs,
            own_jobs: Jobs::default(),
            jobs_latest,
            job_outputs,
            backend,
            clock,
            timers: TimerWheel::new(),
            context: Arc::new(context),
            io: builder.io,
            snapshot_for_queries,
            #[cfg(feature = "testing")]
            effect_log,
            endpoints: JoinSet::new(),
            shutdown_request,
            shutdown_receiver,
            io_inflight: IoInflight::new(),
            shutting_down: false,
            tasks: task_supervisor,
            projections,
            log_publisher: None,
            metrics: builder.metrics,
            metrics_latest: watch::Sender::new(None),
            inbox_step_time,
            inbox_depth,
            runtime_gauges,
        };
        kernel.projections.refresh(&kernel.snapshot);
        for command in startup_commands {
            if kernel
                .dispatch(Delivery {
                    input: Input::Command(command),
                    reply: None,
                    persist_settings: false,
                })
                .await
                .is_some()
            {
                return Err(ServiceError::Build(
                    "the Kernel stopped during startup after repeated Inbox panics".into(),
                ));
            }
        }
        let initial_states = kernel
            .states
            .iter()
            .map(|state| (state.endpoint.project)(&kernel.snapshot))
            .collect();
        kernel.publish_states(initial_states).await;
        kernel.publish_jobs().await;
        kernel
            .publish_job_feedback(encode_job_outputs(
                &kernel.job_outputs,
                &kernel.snapshot,
                kernel.jobs(),
                kernel.jobs(),
            ))
            .await;
        kernel.publish_settings().await;
        kernel.projections.refresh(&kernel.snapshot);
        publish_standard_status(
            &kernel.backend,
            &status_key,
            &status_encoding,
            &status_latest,
        )
        .await;
        let command_sender = CommandSender::new(mpsc::Sender::clone(
            kernel
                .inbox_sender
                .as_ref()
                .expect("the inbox sender exists during startup"),
        ));
        kernel.tasks.start(
            task_specs,
            session,
            command_sender,
            Arc::clone(&kernel.context),
            Arc::clone(&kernel.clock),
        );
        kernel.publish_metrics().await;
        kernel.endpoints.spawn(serve_fixed_reply(
            info_queryable,
            info_key,
            info_payload,
            info_encoding,
        ));
        kernel.endpoints.spawn(serve_state(
            status_queryable,
            status_key,
            status_encoding,
            status_latest.subscribe(),
        ));
        kernel.endpoints.spawn(serve_state(
            metrics_queryable,
            state_key(service, METRICS),
            cdr_encoding(ServiceMetrics::SCHEMA_NAME),
            kernel.metrics_latest.subscribe(),
        ));
        if let Some((queryable, key, encoding, latest)) = pending_settings_serve {
            kernel
                .endpoints
                .spawn(serve_settings(queryable, key, encoding, latest));
        }
        kernel.endpoints.spawn(serve_update_settings(
            update_settings_queryable,
            kernel
                .settings
                .as_ref()
                .map(|endpoint| Arc::clone(&endpoint.driver)),
            mpsc::Sender::clone(
                kernel
                    .inbox_sender
                    .as_ref()
                    .expect("the inbox sender exists during startup"),
            ),
        ));
        for (queryable, into_input) in pending_commands {
            kernel.endpoints.spawn(serve_command(
                queryable,
                into_input,
                mpsc::Sender::clone(
                    kernel
                        .inbox_sender
                        .as_ref()
                        .expect("the inbox sender exists during startup"),
                ),
            ));
        }
        kernel.endpoints.spawn(serve_state(
            jobs_queryable,
            jobs_key(service),
            cdr_encoding(JobList::SCHEMA_NAME),
            kernel.jobs_latest.subscribe(),
        ));
        for (feedback_queryable, feedback_key, feedback, history_queryable, history_key, history) in
            pending_job_outputs
        {
            kernel.endpoints.spawn(serve_state(
                feedback_queryable,
                feedback_key,
                cdr_encoding(JobFeedbackList::SCHEMA_NAME),
                feedback,
            ));
            kernel.endpoints.spawn(serve_state(
                history_queryable,
                history_key,
                cdr_encoding(JobList::SCHEMA_NAME),
                history,
            ));
        }
        for (queryable, key, encoding, latest) in pending_states {
            kernel
                .endpoints
                .spawn(serve_state(queryable, key, encoding, latest));
        }
        for (queryable, answer) in pending_queries {
            kernel.endpoints.spawn(serve_query::<D>(
                queryable,
                answer,
                Arc::clone(&kernel.snapshot_for_queries),
                Arc::clone(&kernel.clock),
            ));
        }
        for (queryable, respond, encoding) in pending_io_queries {
            kernel.endpoints.spawn(
                kernel
                    .metrics
                    .scope(serve_io_query(queryable, respond, encoding)),
            );
        }
        let liveliness_key = service_liveliness_key(service);
        let liveliness = kernel
            .backend
            .declare_liveliness(&liveliness_key)
            .await
            .map_err(|source| ServiceError::DeclareEndpoint {
                key: liveliness_key,
                source,
            })?;
        let liveliness_shutdown = kernel.tasks.shutdown_token();
        let task_spawner = kernel.tasks.spawner();
        task_spawner.spawn(async move {
            hold_liveliness_until_cancelled(liveliness, liveliness_shutdown).await;
        });
        Ok(kernel)
    }

    /// Handles the Commands in the Inbox one at a time, until shutdown finishes or every endpoint has stopped.
    pub async fn run(mut self) -> RunOutcome {
        let mut repeated_inbox_panics = None;
        let mut shutdown_monotonic_deadline = None;
        let mut shutdown_receiver = None;
        core::mem::swap(&mut self.shutdown_receiver, &mut shutdown_receiver);
        let stop_requested = Arc::new(AtomicBool::new(false));
        let stop_flag_for_signals = Arc::clone(&stop_requested);
        let mut signal_shutdown_receiver = shutdown_receiver.clone();
        self.tasks.spawner().spawn(async move {
            wait_for_shutdown_signal(&mut signal_shutdown_receiver).await;
            stop_flag_for_signals.store(true, Ordering::SeqCst);
        });
        let mut metrics_interval =
            tokio::time::interval_at(Instant::now() + METRICS_PERIOD, METRICS_PERIOD);
        metrics_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            if let Some(receiver) = &mut shutdown_receiver
                && *receiver.borrow_and_update()
            {
                stop_requested.store(true, Ordering::SeqCst);
            }
            if stop_requested.load(Ordering::SeqCst) && !self.shutting_down {
                self.begin_shutdown().await;
                shutdown_monotonic_deadline =
                    Some(self.clock.now().monotonic + SHUTDOWN_IO_DRAIN_TIMEOUT);
            }

            if self.shutting_down {
                let remaining = shutdown_monotonic_deadline
                    .map(|deadline| deadline.saturating_sub(self.clock.now().monotonic))
                    .unwrap_or(SHUTDOWN_IO_DRAIN_TIMEOUT);
                while let Ok(Some(delivery)) =
                    tokio::time::timeout(Duration::ZERO, self.inbox.recv()).await
                {
                    if let Some(outcome) = self.dispatch(delivery).await {
                        repeated_inbox_panics = Some(outcome);
                        break;
                    }
                }
                if self.io_inflight.count() == 0 {
                    let task_budget = shutdown_monotonic_deadline
                        .map(|deadline| deadline.saturating_sub(self.clock.now().monotonic))
                        .unwrap_or(SHUTDOWN_IO_DRAIN_TIMEOUT);
                    self.tasks.join_with_budget(task_budget, &self.clock).await;
                    break;
                }
                if remaining == Duration::ZERO {
                    warn!(
                        timeout = ?SHUTDOWN_IO_DRAIN_TIMEOUT,
                        "shutdown io drain timed out"
                    );
                    self.tasks
                        .join_with_budget(Duration::ZERO, &self.clock)
                        .await;
                    break;
                }
                tokio::select! {
                    biased;
                    () = tokio::time::sleep(remaining) => {
                        warn!(
                            timeout = ?SHUTDOWN_IO_DRAIN_TIMEOUT,
                            "shutdown io drain timed out"
                        );
                        self.tasks.join_with_budget(Duration::ZERO, &self.clock).await;
                        break;
                    }
                    delivery = self.inbox.recv() => {
                        if let Some(delivery) = delivery {
                            if let Some(outcome) = self.dispatch(delivery).await {
                                repeated_inbox_panics = Some(outcome);
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                if repeated_inbox_panics.is_some() {
                    break;
                }
                continue;
            }

            if repeated_inbox_panics.is_some() {
                break;
            }

            if self.endpoints.is_empty() {
                break;
            }

            tokio::select! {
                biased;
                _ = async {
                    let Some(receiver) = &mut shutdown_receiver else {
                        pending::<()>().await;
                        return;
                    };
                    while !*receiver.borrow_and_update() {
                        if receiver.changed().await.is_err() {
                            pending::<()>().await;
                        }
                    }
                }, if shutdown_receiver.is_some() => {
                    stop_requested.store(true, Ordering::SeqCst);
                }
                // Polled before the Inbox, so a full Inbox cannot hold back the metrics.
                _ = metrics_interval.tick() => self.publish_metrics().await,
                delivery = self.inbox.recv() => {
                    if let Some(delivery) = delivery {
                        if let Some(outcome) = self.dispatch(delivery).await {
                            repeated_inbox_panics = Some(outcome);
                        }
                    } else if self.endpoints.is_empty() {
                        break;
                    }
                }
                tick = self.timers.next_tick(), if self.timers.waiting() => {
                    if let Some(tick) = tick
                        && let Some(outcome) = self.dispatch(Delivery {
                            input: Input::Command(Command::Tick(tick)),
                            reply: None,
                            persist_settings: false,
                        })
                        .await
                    {
                        repeated_inbox_panics = Some(outcome);
                    }
                }
                _ = self.endpoints.join_next(), if !self.endpoints.is_empty() => {}
            }
            if repeated_inbox_panics.is_some() {
                break;
            }
        }

        self.inbox_sender.take();
        self.endpoints.abort_all();
        while let Ok(Some(delivery)) = tokio::time::timeout(Duration::ZERO, self.inbox.recv()).await
        {
            if let Some(outcome) = self.dispatch(delivery).await {
                repeated_inbox_panics = Some(outcome);
                break;
            }
        }
        if let Some(durable) = &mut self.durable {
            durable.persister.flush_and_shutdown().await;
        }
        if let Some(runtime) = self.log_publisher.take() {
            runtime.shutdown_and_wait().await;
        }
        repeated_inbox_panics.unwrap_or(RunOutcome::Stopped)
    }

    async fn begin_shutdown(&mut self) {
        if self.shutting_down {
            return;
        }
        self.shutting_down = true;
        self.tasks.cancel();
        if let Some(request) = self.shutdown_request.lock().await.take() {
            let delivery = Delivery {
                input: Input::Command(Command::Request(request)),
                reply: None,
                persist_settings: false,
            };
            if let Some(sender) = &self.inbox_sender {
                drop(sender.send(delivery).await);
            }
        }
    }

    /// Applies one Command as a transaction: if the Domain rejects it, or `handle` or a Projection panics, the
    /// Snapshot is restored from a clone taken first and the domain events are dropped.
    ///
    /// Returns [`RunOutcome::RepeatedInboxPanics`] when the rolling panic budget is exhausted (D-29).
    async fn dispatch(&mut self, delivery: Delivery<D>) -> Option<RunOutcome> {
        self.inbox_depth.set(self.inbox.len() as f64);
        let started = self.clock.now().monotonic;
        let unwound = AssertUnwindSafe(self.dispatch_delivery(delivery))
            .catch_unwind()
            .await;
        self.inbox_step_time
            .record(self.clock.now().monotonic.saturating_sub(started));
        match unwound {
            Ok(maybe_stop) => maybe_stop,
            Err(panic) => {
                log_caught_panic(self.service, None, panic);
                if self
                    .tasks
                    .record_inbox_loop_panic(self.clock.now().monotonic)
                    .await
                {
                    Some(RunOutcome::RepeatedInboxPanics)
                } else {
                    None
                }
            }
        }
    }

    async fn dispatch_delivery(&mut self, delivery: Delivery<D>) -> Option<RunOutcome> {
        let Delivery {
            input,
            reply,
            persist_settings,
        } = delivery;
        let job_id = input.job_id();
        if self.shutting_down && reply.is_some() {
            complete_command_reply(
                reply,
                command_ack(job_id, None, Err(Rejection::ShuttingDown)),
            )
            .await;
            return None;
        }
        let now = self.clock.now();
        let backup = self.snapshot.clone();
        let jobs_backup = self.own_jobs.clone();
        let command = match self.accept(input) {
            Ok(command) => command,
            Err(rejection) => {
                self.snapshot = backup;
                self.own_jobs = jobs_backup;
                self.reject(reply, job_id, rejection).await;
                return None;
            }
        };
        let persist_settings = persist_settings && command.is_some();
        let snapshot = &mut self.snapshot;
        let own_jobs = &mut self.own_jobs;
        let jobs_access = self.jobs_access;
        let states = &self.states;
        let job_outputs = &self.job_outputs;
        #[cfg(feature = "testing")]
        let run_effects = self.effect_log.is_none();
        #[cfg(not(feature = "testing"))]
        let run_effects = true;
        let timers = &mut self.timers;
        let io = &self.io;
        let decided = panic::catch_unwind(AssertUnwindSafe(|| {
            let outcome = match command {
                Some(command) => D::handle(snapshot, command, now),
                None => Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                },
            };
            match outcome {
                Outcome::Applied { events, effects } => {
                    apply_sync_effects(&effects, timers, io, run_effects)
                        .map_err(|error| Rejection::Domain(Box::new(error)))?;
                    if let Some(job_id) = job_id {
                        let jobs = jobs_in_mut::<D>(jobs_access, snapshot, own_jobs);
                        if jobs.job(job_id).is_some_and(|job| {
                            !job.nature.lasting && job.status == JobStatus::Executing
                        }) {
                            jobs.end(job_id, JobEnd::Succeeded)?;
                        }
                    }
                    let encoded_states: Vec<_> = states
                        .iter()
                        .map(|state| (state.endpoint.project)(snapshot))
                        .collect();
                    let encoded_job_outputs = encode_job_outputs(
                        job_outputs,
                        snapshot,
                        jobs_in::<D>(jobs_access, snapshot, own_jobs),
                        jobs_in::<D>(jobs_access, &backup, &jobs_backup),
                    );
                    Ok((events, effects, encoded_states, encoded_job_outputs))
                }
                Outcome::Rejected { reason } => Err(Rejection::Domain(reason)),
            }
        }))
        .unwrap_or_else(|panic| {
            log_caught_panic(self.service, Some(inbox_recovery::INBOX_LOOP_NAME), panic);
            Err(Rejection::Panicked)
        });
        match decided {
            Ok((events, effects, encoded_states, encoded_job_outputs)) => {
                {
                    let mut shared = self.snapshot_for_queries.write().await;
                    *shared = self.snapshot.clone();
                }
                #[cfg(feature = "testing")]
                if let Some(log) = &self.effect_log {
                    lock_unpoisoned(log).push(effects.clone());
                }
                if persist_settings && let Some(settings) = &self.settings {
                    let persist_result = lock_unpoisoned(&settings.driver).persist(&self.snapshot);
                    match persist_result {
                        Ok(()) => {
                            lock_unpoisoned(&settings.driver).commit_persisted(&self.snapshot)
                        }
                        Err(error) => {
                            self.snapshot = backup;
                            self.own_jobs = jobs_backup;
                            complete_command_reply(
                                reply,
                                command_ack(job_id, None, Err(Rejection::Domain(error.into()))),
                            )
                            .await;
                            return None;
                        }
                    }
                }
                if let Some(durable) = &self.durable
                    && (durable.changed)(&backup, &self.snapshot)
                {
                    durable
                        .persister
                        .queue_document((durable.serialize)(&self.snapshot));
                }
                self.publish_states(encoded_states).await;
                self.publish_jobs().await;
                let job_results = self.publish_job_feedback(encoded_job_outputs).await;
                self.publish_settings().await;
                self.projections.refresh(&self.snapshot);
                let job = job_id.and_then(|job_id| self.jobs().job(job_id));
                complete_command_reply(reply, command_ack(job_id, job, Ok(()))).await;
                self.publish_events(events).await;
                self.publish_job_results(job_results).await;
                if run_effects {
                    let requests = io_requests::<D>(&effects);
                    if let (Some(inbox_sender), false) = (&self.inbox_sender, requests.is_empty()) {
                        spawn_io_chain(
                            self.tasks.spawner(),
                            self.io.clone(),
                            Arc::clone(&self.context),
                            self.snapshot.clone(),
                            requests,
                            mpsc::Sender::clone(inbox_sender),
                            self.io_inflight.clone(),
                        );
                    }
                }
                self.tasks.mark_inbox_loop_healthy().await;
                None
            }
            Err(rejection) => {
                self.snapshot = backup;
                self.own_jobs = jobs_backup;
                let stop = if matches!(rejection, Rejection::Panicked) {
                    self.tasks
                        .record_inbox_loop_panic(self.clock.now().monotonic)
                        .await
                } else {
                    false
                };
                self.reject(reply, job_id, rejection).await;
                if stop {
                    Some(RunOutcome::RepeatedInboxPanics)
                } else {
                    None
                }
            }
        }
    }

    /// Applies a client's submit or control to the Jobs, and returns the Command the Domain handles next: the Goal
    /// of a Job that executes now, or `None` when only the Jobs changed (D-27).
    fn accept(&mut self, input: Input<D>) -> Result<Option<InboxCommand<D>>, Rejection> {
        match input {
            Input::Command(command) => Ok(Some(command)),
            Input::Submit {
                job_id,
                job_type,
                goal,
                nature,
                request,
            } => match self.jobs_mut().submit(job_id, &job_type, &goal, nature)? {
                Submitted::New if !nature.needs_permission => Ok(Some(Command::Request(request))),
                Submitted::New | Submitted::Retry => Ok(None),
            },
            Input::Control { job_id, control } => {
                self.jobs_mut().control(job_id, control)?;
                if control != (JobControl::AnswerPermission { granted: true }) {
                    return Ok(None);
                }
                let job = self.jobs().job(job_id).ok_or(JobsError::Unknown(job_id))?;
                let decode = self
                    .goal_decoders
                    .get(&job.job_type)
                    .ok_or_else(|| Rejection::UnknownJobType(job.job_type.clone()))?;
                decode(job_id, &job.goal).map(|request| Some(Command::Request(request)))
            }
        }
    }

    /// Replies that the Kernel did not apply a Command, with the status of the Job it named, if that Job is in the
    /// table and its id was not reused.
    async fn reject(
        &self,
        reply: Option<CommandReply>,
        job_id: Option<JobId>,
        rejection: Rejection,
    ) {
        let job = match rejection {
            Rejection::Jobs(JobsError::IdReused(_)) => None,
            _ => job_id.and_then(|job_id| self.jobs().job(job_id)),
        };
        complete_command_reply(reply, command_ack(job_id, job, Err(rejection))).await;
    }

    fn jobs(&self) -> &Jobs {
        jobs_in::<D>(self.jobs_access, &self.snapshot, &self.own_jobs)
    }

    fn jobs_mut(&mut self) -> &mut Jobs {
        jobs_in_mut::<D>(self.jobs_access, &mut self.snapshot, &mut self.own_jobs)
    }

    /// Hands out a [`CommandSender`] while the Inbox is still open.
    pub fn command_sender(&self) -> Option<CommandSender<D>> {
        self.inbox_sender
            .as_ref()
            .map(|sender| CommandSender::new(mpsc::Sender::clone(sender)))
    }

    async fn publish_settings(&self) {
        let Some(settings) = &self.settings else {
            return;
        };
        let sent: Result<(), SendError> = async {
            let payload =
                Bytes::from(lock_unpoisoned(&settings.driver).encode_state(&self.snapshot)?);
            if settings.latest.borrow().as_ref() == Some(&payload) {
                return Ok(());
            }
            let sample = Sample::new(
                settings.key.as_str(),
                Bytes::clone(&payload),
                &settings.encoding,
            );
            self.backend.publish(sample).await?;
            settings.latest.send_replace(Some(payload));
            Ok(())
        }
        .await;
        warn_on_failure("State", &settings.key, sent);
    }

    async fn publish_metrics(&self) {
        self.runtime_gauges.sample();
        let key = state_key(self.service, METRICS);
        let sent: Result<(), SendError> = async {
            let payload = Bytes::from(self.metrics.service_metrics().encode()?);
            if self.metrics_latest.borrow().as_ref() == Some(&payload) {
                return Ok(());
            }
            let encoding = cdr_encoding(ServiceMetrics::SCHEMA_NAME);
            let sample = Sample::new(key.as_str(), Bytes::clone(&payload), &encoding);
            self.backend.publish(sample).await?;
            self.metrics_latest.send_replace(Some(payload));
            Ok(())
        }
        .await;
        warn_on_failure("State", &key, sent);
    }

    async fn publish_jobs(&self) {
        let key = jobs_key(self.service);
        let sent: Result<(), SendError> = async {
            let payload = Bytes::from(job_list(self.jobs()).encode()?);
            if self.jobs_latest.borrow().as_ref() == Some(&payload) {
                return Ok(());
            }
            let encoding = cdr_encoding(JobList::SCHEMA_NAME);
            let sample = Sample::new(key.as_str(), Bytes::clone(&payload), &encoding);
            self.backend.publish(sample).await?;
            self.jobs_latest.send_replace(Some(payload));
            Ok(())
        }
        .await;
        warn_on_failure("State", &key, sent);
    }

    /// Publishes each Job type's Feedback State when it changed and keeps its history for the Query. Returns each
    /// Job type's Job result Events, for [`Self::publish_job_results`] once the Command is acknowledged.
    async fn publish_job_feedback(
        &self,
        outputs: Vec<EncodedJobOutput>,
    ) -> Vec<Vec<Result<Vec<u8>, IdlError>>> {
        let mut job_results = Vec::new();
        for (job_output, encoded) in self.job_outputs.iter().zip(outputs) {
            let EncodedJobOutput {
                feedback,
                history,
                results,
            } = encoded;
            let key = job_feedback_key(self.service, &job_output.job_type);
            let sent: Result<(), SendError> = async {
                let payload = Bytes::from(feedback?);
                if job_output.feedback_latest.borrow().as_ref() == Some(&payload) {
                    return Ok(());
                }
                let encoding = cdr_encoding(JobFeedbackList::SCHEMA_NAME);
                let sample = Sample::new(key.as_str(), Bytes::clone(&payload), &encoding);
                self.backend.publish(sample).await?;
                job_output.feedback_latest.send_replace(Some(payload));
                Ok(())
            }
            .await;
            warn_on_failure("State", &key, sent);
            match history {
                Ok(payload) => {
                    job_output.history.send_replace(Some(Bytes::from(payload)));
                }
                Err(error) => warn_on_failure(
                    "Query",
                    &job_history_key(self.service, &job_output.job_type),
                    Err(error.into()),
                ),
            }
            job_results.push(results);
        }
        job_results
    }

    async fn publish_job_results(&self, job_results: Vec<Vec<Result<Vec<u8>, IdlError>>>) {
        let encoding = cdr_encoding(JobResult::SCHEMA_NAME);
        for (job_output, results) in self.job_outputs.iter().zip(job_results) {
            let key = job_result_key(self.service, &job_output.job_type);
            for encoded in results {
                let sent: Result<(), SendError> = async {
                    let sample = Sample::new(key.as_str(), encoded?, encoding.as_str());
                    self.backend.publish(sample).await?;
                    Ok(())
                }
                .await;
                warn_on_failure("Event", &key, sent);
            }
        }
    }

    async fn publish_states(&self, encoded_states: Vec<Result<Vec<u8>, IdlError>>) {
        for (state, encoded) in self.states.iter().zip(encoded_states) {
            let sent: Result<(), SendError> = async {
                let payload = Bytes::from(encoded?);
                if state.latest.borrow().as_ref() == Some(&payload) {
                    return Ok(());
                }
                let encoding = state.endpoint.encoding.as_str();
                let sample = Sample::new(state.key.as_str(), Bytes::clone(&payload), encoding);
                self.backend.publish(sample).await?;
                state.latest.send_replace(Some(payload));
                Ok(())
            }
            .await;
            warn_on_failure("State", &state.key, sent);
        }
    }

    async fn publish_events(&self, events: Vec<D::Event>) {
        for event in events {
            for endpoint in &self.events {
                if let Some(encoded) = (endpoint.select)(&event) {
                    let key = event_key(self.service, &endpoint.name);
                    let sent: Result<(), SendError> = async {
                        let sample =
                            Sample::new(key.as_str(), encoded?, endpoint.encoding.as_str());
                        self.backend.publish(sample).await?;
                        Ok(())
                    }
                    .await;
                    warn_on_failure("Event", &key, sent);
                }
            }
        }
    }
}

async fn publish_standard_status(
    backend: &Arc<dyn CommsBackend>,
    key: &str,
    encoding: &str,
    latest: &watch::Sender<Option<Bytes>>,
) {
    let status = ServiceStatus {
        status: ServiceStatusStatus::Ready,
        detail: String::new(),
    };
    let sent: Result<(), SendError> = async {
        let payload = Bytes::from(status.encode()?);
        if latest.borrow().as_ref() == Some(&payload) {
            return Ok(());
        }
        let sample = Sample::new(key, Bytes::clone(&payload), encoding);
        backend.publish(sample).await?;
        latest.send_replace(Some(payload));
        Ok(())
    }
    .await;
    warn_on_failure("State", key, sent);
}

/// Answers every `info` query with the same encoded [`ServiceInfo`].
async fn serve_fixed_reply(
    mut queryable: Queryable,
    key: String,
    payload: Bytes,
    encoding: String,
) {
    while let Some(query) = queryable.recv().await {
        let sent = query.reply(Bytes::clone(&payload), encoding.as_str()).await;
        warn_on_failure("Query", &key, sent.map_err(SendError::from));
    }
}

async fn declare(backend: &dyn CommsBackend, key: String) -> Result<Queryable, ServiceError> {
    match backend.declare_queryable(&key).await {
        Ok(queryable) => Ok(queryable),
        Err(source) => Err(ServiceError::DeclareEndpoint { key, source }),
    }
}

/// Reads the Job id from each Command's attachment and decodes its body outside the Inbox loop, so a Command that
/// names no Job, or whose body does not decode, never reaches it.
async fn serve_command<D: Domain>(
    mut queryable: Queryable,
    into_input: IntoInput<D>,
    inbox: mpsc::Sender<Delivery<D>>,
) {
    while let Some(query) = queryable.recv().await {
        let job_id = attached_job_id(&query);
        let body = query
            .body()
            .map(|body| body.payload().to_bytes().into_owned())
            .unwrap_or_default();
        let input = job_id
            .ok_or(Rejection::NoJobId)
            .and_then(|job_id| into_input(job_id, body));
        match input {
            Ok(input) => {
                let delivery = Delivery {
                    input,
                    reply: Some(CommandReply::Query(query)),
                    persist_settings: false,
                };
                drop(inbox.send(delivery).await);
            }
            Err(rejection) => {
                complete_command_reply(
                    Some(CommandReply::Query(query)),
                    command_ack(job_id, None, Err(rejection)),
                )
                .await;
            }
        }
    }
}

/// Decodes `UpdateSettings` into an instant Job, validates the document, and queues it with a flag to persist on
/// success. A Service without settings refuses it.
async fn serve_update_settings<D: Domain>(
    mut queryable: Queryable,
    driver: Option<Arc<Mutex<Box<dyn SettingsDriver<D>>>>>,
    inbox: mpsc::Sender<Delivery<D>>,
) {
    while let Some(query) = queryable.recv().await {
        let job_id = attached_job_id(&query);
        let goal = query
            .body()
            .map(|body| body.payload().to_bytes().into_owned())
            .unwrap_or_default();
        let input = job_id.ok_or(Rejection::NoJobId).and_then(|job_id| {
            let driver = driver.as_ref().ok_or(Rejection::NoSettings)?;
            let UpdateSettingsGoal { envelope } =
                UpdateSettingsGoal::decode(&goal).map_err(Rejection::InvalidBody)?;
            let request = lock_unpoisoned(driver)
                .request_from_envelope(envelope)
                .map_err(Rejection::Domain)?;
            Ok(Input::Submit {
                job_id,
                job_type: UPDATE_SETTINGS.to_owned(),
                goal,
                nature: JobNature::INSTANT,
                request,
            })
        });
        match input {
            Ok(input) => {
                let delivery = Delivery {
                    input,
                    reply: Some(CommandReply::Query(query)),
                    persist_settings: true,
                };
                drop(inbox.send(delivery).await);
            }
            Err(rejection) => {
                complete_command_reply(
                    Some(CommandReply::Query(query)),
                    command_ack(job_id, None, Err(rejection)),
                )
                .await;
            }
        }
    }
}

/// The id of the Job a client's Command names in its attachment, if it is one.
fn attached_job_id(query: &Query) -> Option<JobId> {
    let attachment = query.body().and_then(QueryBody::attachment)?;
    core::str::from_utf8(&attachment.to_bytes())
        .ok()?
        .parse()
        .ok()
}

/// Answers every get on the `settings` State with the last value the backbone accepted.
async fn serve_settings(
    mut queryable: Queryable,
    key: String,
    encoding: String,
    latest: watch::Receiver<Option<Bytes>>,
) {
    while let Some(query) = queryable.recv().await {
        let current = latest.borrow().clone();
        if let Some(payload) = current {
            let sent = query.reply(payload, encoding.as_str()).await;
            warn_on_failure("State", &key, sent.map_err(SendError::from));
        }
    }
}

/// Answers every get on a Domain Query endpoint from the current Snapshot.
async fn serve_query<D: Domain>(
    mut queryable: Queryable,
    answer: AnswerQuery<D>,
    snapshot: Arc<tokio::sync::RwLock<D::Snapshot>>,
    clock: Arc<dyn Clock>,
) {
    while let Some(query) = queryable.recv().await {
        let body = query
            .body()
            .map(|body| body.payload().to_bytes())
            .unwrap_or_default();
        let now = clock.now();
        let answered = {
            let shared = snapshot.read().await;
            panic::catch_unwind(AssertUnwindSafe(|| answer(&shared, &body, now)))
                .unwrap_or(Err(Unanswered::Panicked))
        };
        reply(query, answered).await;
    }
}

/// Answers each IO query in turn, so one slow answer delays the next instead of running beside it.
async fn serve_io_query(mut queryable: Queryable, respond: Respond, encoding: String) {
    while let Some(query) = queryable.recv().await {
        let body = query
            .body()
            .map(|body| body.payload().to_bytes().into_owned());
        let answered = AssertUnwindSafe(respond(body.unwrap_or_default()))
            .catch_unwind()
            .await
            .unwrap_or(Err(Unanswered::Panicked));
        reply(query, answered.map(|payload| (payload, encoding.clone()))).await;
    }
}

async fn serve_state(
    mut queryable: Queryable,
    key: String,
    encoding: String,
    latest: watch::Receiver<Option<Bytes>>,
) {
    while let Some(query) = queryable.recv().await {
        let current = latest.borrow().clone();
        if let Some(payload) = current {
            let sent = query.reply(payload, encoding.as_str()).await;
            warn_on_failure("State", &key, sent.map_err(SendError::from));
        }
    }
}

async fn complete_command_reply(reply: Option<CommandReply>, ack: CommandAck) {
    let Some(reply) = reply else {
        return;
    };
    match reply {
        CommandReply::Ack(sender) => {
            drop(sender.send(ack));
        }
        CommandReply::Query(query) => {
            let key = query.key_expression().to_owned();
            let sent: Result<(), SendError> = async {
                let encoding = cdr_encoding(CommandAck::SCHEMA_NAME);
                query.reply(ack.encode()?, encoding).await?;
                Ok(())
            }
            .await;
            warn_on_failure("CommandAck", &key, sent);
        }
    }
}

/// Sends a Query's answer, or an error reply whose payload is the reason it got none.
async fn reply(query: Query, answered: Result<(Vec<u8>, String), Unanswered>) {
    let key = query.key_expression().to_owned();
    let sent = match answered {
        Ok((payload, encoding)) => query.reply(payload, encoding).await,
        Err(unanswered) => {
            let reason = unanswered.to_string().into_bytes();
            query.reply_error(reason, REASON_ENCODING).await
        }
    };
    warn_on_failure("Query", &key, sent.map_err(SendError::from));
}

/// A failed publish or reply is logged and never stops the Inbox loop.
fn warn_on_failure(kind: &'static str, key: &str, sent: Result<(), SendError>) {
    if let Err(error) = sent {
        warn!(%error, kind, key, "Failed to send");
    }
}

/// The schema text of a Job output key as `info` lists it: the wrapper `M` on the wire, then the part its bytes
/// carry, under the `MSG: <package>/<Name>` line ROS 2 gives a dependency. A Job type without the part gets `M`
/// alone.
fn carrying<M: Message>(part: MessageType) -> String {
    match (part.name.split_once('/'), part.name.rsplit_once('/')) {
        (Some((package, _)), Some((_, name))) => format!(
            "{}\n{SCHEMA_SEPARATOR}\nMSG: {package}/{name}\n{}",
            M::SCHEMA,
            part.schema
        ),
        _ => M::SCHEMA.to_owned(),
    }
}

/// Encodes each Job type's Feedback State and history from `snapshot` and `jobs`, and the Job result of each Job
/// that ended since `before`.
fn encode_job_outputs<D: Domain>(
    job_outputs: &[JobTypeOutput<D>],
    snapshot: &D::Snapshot,
    jobs: &Jobs,
    before: &Jobs,
) -> Vec<EncodedJobOutput> {
    job_outputs
        .iter()
        .map(|job_output| {
            let of_type = || {
                jobs.list()
                    .filter(|job| job.job_type == job_output.job_type)
            };
            let fed_back = job_output.output.feedback.as_ref().map_or_else(
                || Ok(Vec::new()),
                |project| {
                    of_type()
                        .filter(|job| !job.status.has_ended())
                        .filter_map(|job| {
                            let encoded = project(snapshot, job.job_id)?;
                            Some(encoded.map(|feedback| JobFeedback {
                                job_id: job.job_id.to_string(),
                                feedback,
                            }))
                        })
                        .collect()
                },
            );
            let history = JobList {
                jobs: of_type()
                    .filter(|job| job.status.has_ended())
                    .map(job_status)
                    .collect(),
            };
            let results = of_type()
                .filter(|job| {
                    job.status.has_ended()
                        && !before
                            .job(job.job_id)
                            .is_some_and(|earlier| earlier.status.has_ended())
                })
                .map(|job| {
                    let result = job_output
                        .output
                        .result
                        .as_ref()
                        .map_or_else(|| Ok(Vec::new()), |result| result(snapshot, job.job_id))?;
                    JobResult {
                        job: job_status(job),
                        result,
                    }
                    .encode()
                })
                .collect();
            EncodedJobOutput {
                feedback: fed_back.and_then(|entries| JobFeedbackList { jobs: entries }.encode()),
                history: history.encode(),
                results,
            }
        })
        .collect()
}

/// The Jobs, in the Snapshot when `jobs_access` says the Domain keeps them there, else in `own_jobs`.
fn jobs_in<'jobs, D: Domain>(
    jobs_access: Option<JobsAccess<D>>,
    snapshot: &'jobs D::Snapshot,
    own_jobs: &'jobs Jobs,
) -> &'jobs Jobs {
    match jobs_access {
        Some((jobs, _jobs_mut)) => jobs(snapshot),
        None => own_jobs,
    }
}

/// The Jobs, in the Snapshot when `jobs_access` says the Domain keeps them there, else in `own_jobs`.
fn jobs_in_mut<'jobs, D: Domain>(
    jobs_access: Option<JobsAccess<D>>,
    snapshot: &'jobs mut D::Snapshot,
    own_jobs: &'jobs mut Jobs,
) -> &'jobs mut Jobs {
    match jobs_access {
        Some((_jobs, jobs_mut)) => jobs_mut(snapshot),
        None => own_jobs,
    }
}
