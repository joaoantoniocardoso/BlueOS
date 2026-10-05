//! What a Service's `build` declares: the initial Snapshot and how the Domain meets the backbone.

use core::{error::Error, future::Future, num::NonZeroU32, pin::Pin};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tokio::sync::watch;

use blueos_api::{Message, cdr_encoding};
use blueos_domain::{Command, Domain, DomainDurable, DomainQueries, IoError};
use blueos_idl::{
    Error as IdlError,
    msg::blueos_msgs::{EndpointInfo, JobList, JobStatus, JobStatusStatus, SettingsEnvelope},
};
use blueos_jobs::{DomainJobs, Job, JobId, JobNature, Jobs};
use blueos_settings::SettingsSchema;

use crate::{
    durable_state::{
        DurableStateRegistration, register_durable_state, register_durable_state_with_jobs,
    },
    kernel::{Rejection, Unanswered, io::IoExecutors},
    service::{Service, ServiceContext},
    settings::{SettingsRegistration, register_settings},
    shutdown::{ShutdownHandle, new_shutdown_channel},
    tasks::{RestartPolicy, TaskContext, TaskFailed, TaskSpec},
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
    pub(crate) projections: Vec<Box<dyn crate::projection::RefreshProjection<D> + Send + Sync>>,
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
        }
    }
}

impl<D: DomainDurable, Context> ServiceBuilder<D, Context> {
    /// Persists the Domain's durable Snapshot field as versioned JSON next to the settings (D-28).
    pub fn durable_state(mut self, version: NonZeroU32) -> Self
    where
        D::DurableState: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
    {
        self.durable = Some(DurableStateDeclaration {
            open: register_durable_state::<D>,
            version,
        });
        self
    }
}

impl<D, Context> ServiceBuilder<D, Context>
where
    D: DomainDurable + DomainJobs,
    D::DurableState: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
{
    /// Like [`ServiceBuilder::durable_state`], and persists the Domain's Jobs with the durable part.
    pub fn durable_state_with_jobs(mut self, version: NonZeroU32) -> Self {
        self.jobs = Some((D::jobs, D::jobs_mut));
        self.durable = Some(DurableStateDeclaration {
            open: register_durable_state_with_jobs::<D>,
            version,
        });
        self
    }
}

impl<D: DomainJobs, Context> ServiceBuilder<D, Context> {
    /// Adds the Job type `name` with its `nature` (D-36). A client submits a Job on `command/<name>` with an `M` as
    /// its Goal, which `into_request` turns into the Domain's Request together with the Job's id, so the Domain can
    /// end the Job with [`Jobs::end`]. A Goal that does not decode, or that `into_request` refuses, is rejected
    /// before it reaches the Inbox, with the refusal as the reason. The Kernel keeps this Service's Jobs in the
    /// Snapshot.
    pub fn job<M: Message + 'static>(
        mut self,
        name: &str,
        nature: JobNature,
        into_request: impl Fn(JobId, M) -> Result<D::Request, Refusal> + Send + Sync + 'static,
    ) -> Self {
        self.jobs = Some((D::jobs, D::jobs_mut));
        self.commands.push(CommandEndpoint {
            name: name.to_owned(),
            nature,
            decode: Arc::new(move |job_id, body| {
                into_request(job_id, M::decode(body).map_err(Rejection::InvalidBody)?)
                    .map_err(Rejection::Refused)
            }),
        });
        self
    }
}

impl<D: Domain + DomainQueries, Context> ServiceBuilder<D, Context> {
    /// Adds the Query endpoint `name`. Its body is an `M`, which `into_query` turns into the Domain's Query; the
    /// answer is the `R` that `into_response` makes of the Domain's Response from the current Snapshot.
    /// `into_response` returns `None` for a Response that does not belong to this endpoint. A body that does not
    /// decode, a refusal, a `None` or a panic is replied as an error with its reason.
    pub fn query<M: Message + 'static, R: Message + 'static>(
        mut self,
        name: &str,
        into_query: impl Fn(M) -> Result<D::Query, Refusal> + Send + Sync + 'static,
        into_response: impl Fn(D::Response) -> Option<R> + Send + Sync + 'static,
    ) -> Self {
        let encoding = cdr_encoding(R::SCHEMA_NAME);
        self.queries.push((
            name.to_owned(),
            Arc::new(move |snapshot, body, now| {
                let query = into_query(M::decode(body).map_err(Unanswered::InvalidBody)?)
                    .map_err(Unanswered::Refused)?;
                let response = into_response(D::query(snapshot, query, now))
                    .ok_or(Unanswered::OtherResponse)?;
                Ok((
                    response.encode().map_err(Unanswered::Encode)?,
                    encoding.clone(),
                ))
            }),
        ));
        self
    }
}

impl<D: Domain, Context> ServiceBuilder<D, Context> {
    /// Takes the version, build label and capabilities `info` publishes from `S`, and the folder the settings and the
    /// durable state live in from `service` (D-25). The Kernel's entry points call it after `build`, so `build`
    /// never passes them; a test that starts a [`Kernel`](crate::Kernel) by hand calls it too.
    pub fn for_service<S: Service<Domain = D>>(
        mut self,
        service: &ServiceContext<S::Arguments>,
    ) -> Self {
        self.metadata = ServiceMetadata {
            version: S::VERSION,
            build: S::BUILD,
            capabilities: S::CAPABILITIES,
        };
        self.settings_folder = service.settings_path().map(PathBuf::from);
        self
    }

    /// Endpoints from the Service manifest, listed in `ServiceInfo` on the `info` query. The generated `register`
    /// sets this; the Kernel adds the standard endpoints separately.
    pub fn manifest_endpoints(mut self, manifest_endpoints: Vec<EndpointInfo>) -> Self {
        self.manifest_endpoints = manifest_endpoints;
        self
    }

    /// Domain Command dispatched through the Inbox once startup finishes, in registration order.
    ///
    /// Use this instead of querying the service's own command keys at startup: those queryables are not served until
    /// after the initial States are published and the liveliness token is declared.
    pub fn on_start(mut self, request: D::Request) -> Self {
        self.startup_commands.push(Command::Request(request));
        self
    }

    /// Domain Command dispatched on `SIGINT`, `SIGTERM`, or [`ShutdownHandle::trigger`].
    pub fn on_shutdown(mut self, request: D::Request) -> Self {
        self.shutdown_request = Some(request);
        self
    }

    /// Handle for requesting graceful shutdown in tests (no real signals).
    pub fn shutdown_handle(&mut self) -> ShutdownHandle {
        if let Some(sender) = &self.shutdown_sender {
            return ShutdownHandle::new(sender.clone());
        }
        let (handle, receiver) = new_shutdown_channel();
        self.shutdown_sender = Some(handle.sender());
        self.shutdown_receiver = Some(receiver);
        handle
    }

    /// The executor for every [`Effect::Io`]. It returns an optional IO result Command, or an [`IoError`] the Kernel
    /// turns into [`Domain::io_failed`].
    pub fn io<F, Fut>(mut self, executor: F) -> Self
    where
        F: Fn(&Context, &D::Snapshot, D::IoRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<D::IoResult>, IoError>> + Send + 'static,
    {
        self.io.r#async = Some(Arc::new(move |context, snapshot, request| {
            Box::pin(executor(context, snapshot, request))
        }));
        self
    }

    /// The executor for IO the Domain marks with [`Domain::io_runs_on_blocking_thread`]. The Kernel runs it with
    /// [`tokio::task::spawn_blocking`], with the same ordering and result reporting as [`.io`](Self::io).
    pub fn blocking_io<F>(mut self, executor: F) -> Self
    where
        F: Fn(&Context, &D::Snapshot, D::IoRequest) -> Result<Option<D::IoResult>, IoError>
            + Send
            + Sync
            + 'static,
    {
        self.io.blocking = Some(Arc::new(executor));
        self
    }

    /// Registers Python-compatible settings (D-11): the Kernel loads once at startup, owns `UpdateSettings`, and
    /// persists after each successful update.
    pub fn settings<S>(
        mut self,
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
            into_snapshot,
            from_snapshot,
            into_request,
        ));
        self
    }

    /// Adds the instant Job type `name`, whose Jobs succeed in the step that executes them (D-36). A client
    /// submits a Job on `command/<name>` with an `M` as its Goal, which `into_request` turns into the Domain's
    /// Request. A Goal that does not decode, or that `into_request` refuses, is rejected before it reaches the
    /// Inbox, with the refusal as the reason.
    pub fn command<M: Message + 'static>(
        mut self,
        name: &str,
        into_request: impl Fn(M) -> Result<D::Request, Refusal> + Send + Sync + 'static,
    ) -> Self {
        self.commands.push(CommandEndpoint {
            name: name.to_owned(),
            nature: JobNature::INSTANT,
            decode: Arc::new(move |_job_id, body| {
                into_request(M::decode(body).map_err(Rejection::InvalidBody)?)
                    .map_err(Rejection::Refused)
            }),
        });
        self
    }

    /// Publishes the Feedback of the Job type `job_type` on its `jobs/<job_type>/feedback` State (D-12, D-36). After
    /// every applied Command, the State lists, for each active Job of that type, the `M` that `feedback` makes of the
    /// Snapshot, and leaves out a Job that `feedback` returns `None` for. A Job leaves it when it ends. `feedback`
    /// must be pure, like a State's projection: a panic in it restores the Snapshot and rejects the Command.
    pub fn job_feedback<M: Message + 'static>(
        mut self,
        job_type: &str,
        feedback: impl Fn(&D::Snapshot, JobId) -> Option<M> + Send + Sync + 'static,
    ) -> Self {
        let job_output = self.job_output(job_type);
        job_output.feedback = Some(Box::new(move |snapshot, job_id| {
            feedback(snapshot, job_id).map(|message| message.encode())
        }));
        job_output.feedback_type = MessageType {
            name: M::SCHEMA_NAME,
            schema: M::SCHEMA,
        };
        self
    }

    /// Publishes the Job result of the Job type `job_type` on its `jobs/<job_type>/result` Event when a Job ends, the
    /// Kernel ending it included (D-12, D-36): the `M` that `result` makes of the Snapshot after the step that ended
    /// the Job, so the Snapshot must still hold what `result` needs in that step. `result` must be pure.
    pub fn job_result<M: Message + 'static>(
        mut self,
        job_type: &str,
        result: impl Fn(&D::Snapshot, JobId) -> M + Send + Sync + 'static,
    ) -> Self {
        let job_output = self.job_output(job_type);
        job_output.result = Some(Box::new(move |snapshot, job_id| {
            result(snapshot, job_id).encode()
        }));
        job_output.result_type = MessageType {
            name: M::SCHEMA_NAME,
            schema: M::SCHEMA,
        };
        self
    }

    /// Declares a Projection for Tasks: a pure function of the Snapshot, recomputed after every applied Command
    /// and delivered on a deduplicated typed `watch` receiver. It is not published on the backbone.
    pub fn projection<T>(
        mut self,
        project: impl Fn(&D::Snapshot) -> T + Send + Sync + 'static,
    ) -> (Self, crate::projection::Projection<T>)
    where
        T: Clone + PartialEq + Send + Sync + 'static,
    {
        let (handle, refresh) = crate::projection::register_projection(project, &self.snapshot);
        self.projections.push(refresh);
        (self, handle)
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

    /// Adds the IO query endpoint `name`, answered by `respond` outside the Inbox, one query at a time: for reads
    /// that need IO but no Domain state. A body that does not decode, a refusal or a panic in `respond` is replied
    /// as an error with its reason.
    pub fn io_query<Q: Message + 'static, R: Message + 'static>(
        mut self,
        name: &str,
        respond: impl Fn(Q) -> Pin<Box<dyn Future<Output = Result<R, Refusal>> + Send>>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        let respond = Arc::new(respond);
        self.io_queries.push(IoQueryEndpoint {
            name: name.to_owned(),
            encoding: cdr_encoding(R::SCHEMA_NAME),
            respond: Box::new(move |body| {
                let respond = Arc::clone(&respond);
                Box::pin(async move {
                    let request = Q::decode(&body).map_err(Unanswered::InvalidBody)?;
                    let response = respond(request).await.map_err(Unanswered::Refused)?;
                    response.encode().map_err(Unanswered::Encode)
                })
            }),
        });
        self
    }

    /// Declares a long-running Task supervised by the Kernel (D-27).
    pub fn task<F, Fut>(mut self, name: &str, policy: RestartPolicy, run: F) -> Self
    where
        F: Fn(TaskContext<D, Context>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), TaskFailed>> + Send + 'static,
    {
        self.tasks.push(TaskSpec {
            name: name.to_owned(),
            policy,
            run: Arc::new(move |context| Box::pin(run(context))),
        });
        self
    }

    fn job_output(&mut self, job_type: &str) -> &mut JobOutput<D> {
        self.job_outputs.entry(job_type.to_owned()).or_default()
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

/// Every Job as clients see it in the `jobs` State.
pub(crate) fn job_list(jobs: &Jobs) -> JobList {
    JobList {
        jobs: jobs.list().map(job_status).collect(),
    }
}

/// One Job as clients see it in the `jobs` State, a Job result and a history.
pub(crate) fn job_status(job: &Job) -> JobStatus {
    JobStatus {
        job_id: job.job_id.to_string(),
        job_type: job.job_type.clone(),
        status: wire_status(job.status),
        reason: job.reason.clone(),
    }
}

/// The status of `blueos_msgs/JobStatus` for a Job's status; `blueos_msgs/CommandAck` shares its `STATUS_` values.
pub(crate) const fn wire_status(status: blueos_jobs::JobStatus) -> JobStatusStatus {
    match status {
        blueos_jobs::JobStatus::Accepted => JobStatusStatus::Accepted,
        blueos_jobs::JobStatus::Executing => JobStatusStatus::Executing,
        blueos_jobs::JobStatus::Canceling => JobStatusStatus::Canceling,
        blueos_jobs::JobStatus::Succeeded => JobStatusStatus::Succeeded,
        blueos_jobs::JobStatus::Canceled => JobStatusStatus::Canceled,
        blueos_jobs::JobStatus::Aborted => JobStatusStatus::Aborted,
        blueos_jobs::JobStatus::WaitingForPermission => JobStatusStatus::WaitingForPermission,
        blueos_jobs::JobStatus::WaitingForResource => JobStatusStatus::WaitingForResource,
        blueos_jobs::JobStatus::Paused => JobStatusStatus::Paused,
    }
}
