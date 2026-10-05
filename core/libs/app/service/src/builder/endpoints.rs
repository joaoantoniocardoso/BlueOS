//! Durable state, Jobs, Queries, and endpoint wiring on `ServiceBuilder`.

use core::num::NonZeroU32;
use std::sync::Arc;

use blueos_api::{Message, cdr_encoding};
use blueos_domain::{Domain, DomainDurable, DomainQueries};
use blueos_jobs::{DomainJobs, JobId, JobNature};

use crate::{
    durable_state::{register_durable_state, register_durable_state_with_jobs},
    kernel::{Rejection, Unanswered},
};

use super::types::{CommandEndpoint, DurableStateDeclaration, Refusal, ServiceBuilder};

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
