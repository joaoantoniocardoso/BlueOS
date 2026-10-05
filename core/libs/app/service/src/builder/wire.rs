//! Command, state, event, and task registration.

use core::{future::Future, pin::Pin};
use std::sync::Arc;

use blueos_api::{Message, cdr_encoding};
use blueos_domain::Domain;
use blueos_jobs::{JobId, JobNature};

use crate::{
    kernel::{Rejection, Unanswered},
    tasks::{RestartPolicy, TaskContext, TaskFailed, TaskSpec},
};

use super::types::{
    CommandEndpoint, EventEndpoint, IoQueryEndpoint, JobOutput, MessageType, Refusal,
    ServiceBuilder, StateEndpoint,
};

impl<D: Domain, Context> ServiceBuilder<D, Context> {
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
        let (handle, refresh) =
            crate::projection::register_projection::<D, T>(project, &self.snapshot);
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
    // qual:test_helper
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
