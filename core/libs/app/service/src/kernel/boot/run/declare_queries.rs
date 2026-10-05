//! Declares domain state and query endpoints from the builder.

use std::sync::Arc;

use blueos_api::query_key;
use blueos_comms::CommsBackend;
use blueos_domain::Domain;

use crate::{
    builder::{AnswerQuery, ServiceBuilder},
    projection::ProjectionRegistry,
    service::ServiceError,
};

use super::super::{
    super::{
        endpoints::declare,
        types::{Kernel, PublishedState},
    },
    states::declare_domain_states,
};

pub(super) struct QueryEndpointsBootPrepared<D: Domain> {
    pub states: Vec<PublishedState<D>>,
    pub pending_states: Vec<(
        blueos_comms::Queryable,
        String,
        String,
        tokio::sync::watch::Receiver<Option<bytes::Bytes>>,
    )>,
    pub pending_queries: Vec<(blueos_comms::Queryable, AnswerQuery<D>)>,
    pub pending_io_queries: Vec<(blueos_comms::Queryable, crate::builder::Respond, String)>,
    pub projections: ProjectionRegistry<D>,
}

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    pub(super) async fn declare_state_and_query_endpoints_at_boot(
        service: &'static str,
        backend: Arc<dyn CommsBackend>,
        builder: &mut ServiceBuilder<D, Context>,
    ) -> Result<QueryEndpointsBootPrepared<D>, ServiceError> {
        let state_endpoints = core::mem::take(&mut builder.states)
            .into_iter()
            .map(|endpoint| (blueos_api::state_key(service, &endpoint.name), endpoint));
        let (states, pending_states) =
            declare_domain_states(backend.as_ref(), state_endpoints).await?;
        let mut pending_queries = Vec::new();
        for (name, answer) in core::mem::take(&mut builder.queries) {
            let queryable = declare(&*backend, query_key(service, &name)).await?;
            pending_queries.push((queryable, answer));
        }
        let mut pending_io_queries = Vec::new();
        for endpoint in core::mem::take(&mut builder.io_queries) {
            let queryable = declare(&*backend, query_key(service, &endpoint.name)).await?;
            pending_io_queries.push((queryable, endpoint.respond, endpoint.encoding));
        }
        let projections = ProjectionRegistry::new(core::mem::take(&mut builder.projections));
        Ok(QueryEndpointsBootPrepared {
            states,
            pending_states,
            pending_queries,
            pending_io_queries,
            projections,
        })
    }
}
