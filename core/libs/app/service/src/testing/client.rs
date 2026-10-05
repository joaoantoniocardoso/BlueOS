//! Harness client: backbone submit, query, and late reads.

use blueos_api::{
    CommandAck, Message, cdr_encoding, command_key, info_query_key, job_feedback_key,
    job_history_key, jobs_key, query_key, settings_key, state_key,
};
use blueos_comms::{QueryBody, ReplyError};
use blueos_idl::msg::{
    blueos_msgs::{JobFeedbackList, JobList, PermissionAnswer, ServiceInfo},
    std_msgs::Empty,
};
use blueos_jobs::{JobControl, JobId};

use crate::{Service, new_job_id};

use super::{Harness, HarnessError, REPLY_TIMEOUT};

impl<S: Service> Harness<S> {
    /// Submits a Job of type `command` with `goal` and a new Job id, as a client would, and returns the ack.
    ///
    /// # Errors
    ///
    /// When the Service does not reply exactly once with a [`CommandAck`].
    pub async fn send<M: Message>(
        &self,
        command: &str,
        goal: &M,
    ) -> Result<CommandAck, HarnessError> {
        self.submit(command, new_job_id(), goal).await
    }

    /// Submits the Job `job_id` of type `job_type` with `goal`, as a client would, and returns the ack.
    ///
    /// # Errors
    ///
    /// When the Service does not reply exactly once with a [`CommandAck`].
    pub async fn submit<M: Message>(
        &self,
        job_type: &str,
        job_id: JobId,
        goal: &M,
    ) -> Result<CommandAck, HarnessError> {
        let body = QueryBody::new(goal.encode()?, cdr_encoding(M::SCHEMA_NAME));
        self.command(job_type, job_id, body).await
    }

    /// Sends `control` for the Job `job_id`, as a client would, and returns the ack.
    ///
    /// # Errors
    ///
    /// When the Service does not reply exactly once with a [`CommandAck`].
    pub async fn control(
        &self,
        job_id: JobId,
        control: JobControl,
    ) -> Result<CommandAck, HarnessError> {
        let body = match control {
            JobControl::AnswerPermission { granted } => QueryBody::new(
                PermissionAnswer { granted }.encode()?,
                cdr_encoding(PermissionAnswer::SCHEMA_NAME),
            ),
            JobControl::Cancel | JobControl::Pause | JobControl::Resume => {
                QueryBody::new(Empty {}.encode()?, cdr_encoding(Empty::SCHEMA_NAME))
            }
        };
        self.command(&control.to_string(), job_id, body).await
    }

    async fn command(
        &self,
        command: &str,
        job_id: JobId,
        body: QueryBody,
    ) -> Result<CommandAck, HarnessError> {
        let key = command_key(S::NAME, command);
        let body = body.with_attachment(job_id.to_string().into_bytes());
        let replies = self
            .backend
            .get(&key, Some(body), REPLY_TIMEOUT)
            .await
            .map_err(|error| HarnessError::Comms {
                key: key.clone(),
                source: error,
            })?;
        let [Ok(reply)] = replies.as_slice() else {
            return Err(HarnessError::CommandAck {
                command: command.to_owned(),
                count: replies.len(),
            });
        };
        CommandAck::decode(&reply.payload().to_bytes()).map_err(|error| HarnessError::Decode {
            key,
            schema: CommandAck::SCHEMA_NAME,
            error,
        })
    }

    /// Sends `request` to the Query or IO query endpoint `query`, as a client would, and returns the answer, or the
    /// error reply.
    ///
    /// # Errors
    ///
    /// When the Service does not reply exactly once, or replies with something other than an `R`.
    pub async fn query<Q: Message, R: Message>(
        &self,
        query: &str,
        request: &Q,
    ) -> Result<Result<R, ReplyError>, HarnessError> {
        let key = query_key(S::NAME, query);
        let body = QueryBody::new(request.encode()?, cdr_encoding(Q::SCHEMA_NAME));
        let replies = self
            .backend
            .get(&key, Some(body), REPLY_TIMEOUT)
            .await
            .map_err(|error| HarnessError::Comms {
                key: key.clone(),
                source: error,
            })?;
        let [reply] = replies.as_slice() else {
            return Err(HarnessError::ReplyCount {
                key,
                count: replies.len(),
            });
        };
        match reply {
            Ok(sample) => {
                let decoded = R::decode(&sample.payload().to_bytes()).map_err(|error| {
                    HarnessError::Decode {
                        key: query_key(S::NAME, query),
                        schema: R::SCHEMA_NAME,
                        error,
                    }
                })?;
                Ok(Ok(decoded))
            }
            Err(error) => Ok(Err(error.clone())),
        }
    }

    /// Asks the standard `info` Query with no body, as the inspector does, and returns the endpoints it lists.
    ///
    /// # Errors
    ///
    /// When the Service does not reply exactly once with a [`ServiceInfo`].
    // qual:test_helper
    pub async fn info(&self) -> Result<ServiceInfo, HarnessError> {
        self.read_one(&info_query_key(S::NAME)).await
    }

    /// Reads the State `state`, as a late client would.
    ///
    /// # Errors
    ///
    /// When the Service does not reply exactly once with an `M`.
    pub async fn state<M: Message>(&self, state: &str) -> Result<M, HarnessError> {
        self.read_one(&state_key(S::NAME, state)).await
    }

    /// Reads the standard `settings` State, as a late client would.
    ///
    /// # Errors
    ///
    /// When the Service does not reply exactly once with an `M`.
    pub async fn settings<M: Message>(&self) -> Result<M, HarnessError> {
        self.read_one(&settings_key(S::NAME)).await
    }

    /// Reads the standard `jobs` State, as a late client would.
    ///
    /// # Errors
    ///
    /// When the Service does not reply exactly once with a [`JobList`].
    pub async fn jobs(&self) -> Result<JobList, HarnessError> {
        self.read_one(&jobs_key(S::NAME)).await
    }

    /// Reads the Feedback State of the Job type `job_type`, as a late client would.
    ///
    /// # Errors
    ///
    /// When the Service does not reply exactly once with a [`JobFeedbackList`].
    pub async fn job_feedback(&self, job_type: &str) -> Result<JobFeedbackList, HarnessError> {
        self.read_one(&job_feedback_key(S::NAME, job_type)).await
    }

    /// Calls the history Query of the Job type `job_type`: its last finished Jobs, in the order they ended.
    ///
    /// # Errors
    ///
    /// When the Service does not reply exactly once with a [`JobList`].
    // qual:test_helper
    pub async fn job_history(&self, job_type: &str) -> Result<JobList, HarnessError> {
        self.read_one(&job_history_key(S::NAME, job_type)).await
    }

    /// Asks `key` with no body, as a late client would, and decodes the one reply it expects.
    async fn read_one<M: Message>(&self, key: &str) -> Result<M, HarnessError> {
        let replies = self
            .backend
            .get(key, None, REPLY_TIMEOUT)
            .await
            .map_err(|error| HarnessError::Comms {
                key: key.to_owned(),
                source: error,
            })?;
        let [Ok(reply)] = replies.as_slice() else {
            return Err(HarnessError::ReplyCount {
                key: key.to_owned(),
                count: replies.len(),
            });
        };
        M::decode(&reply.payload().to_bytes()).map_err(|error| HarnessError::Decode {
            key: key.to_owned(),
            schema: M::SCHEMA_NAME,
            error,
        })
    }
}
