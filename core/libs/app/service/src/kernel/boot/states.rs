//! Domain State endpoint declaration during Kernel startup.

use tokio::sync::watch;

use blueos_comms::{CommsBackend, Queryable};
use blueos_domain::Domain;

use crate::{builder::StateEndpoint, service::ServiceError};

use super::super::{endpoints::declare, types::PublishedState};

pub(crate) async fn declare_domain_states<D: Domain>(
    backend: &dyn CommsBackend,
    state_endpoints: impl Iterator<Item = (String, StateEndpoint<D>)>,
) -> Result<
    (
        Vec<PublishedState<D>>,
        Vec<(
            Queryable,
            String,
            String,
            watch::Receiver<Option<bytes::Bytes>>,
        )>,
    ),
    ServiceError,
> {
    let mut states = Vec::new();
    let mut pending = Vec::new();
    for (key, endpoint) in state_endpoints {
        let (published, pending_entry) = declare_one_state(backend, key, endpoint).await?;
        states.push(published);
        pending.push(pending_entry);
    }
    Ok((states, pending))
}

async fn declare_one_state<D: Domain>(
    backend: &dyn CommsBackend,
    key: String,
    endpoint: StateEndpoint<D>,
) -> Result<
    (
        PublishedState<D>,
        (
            Queryable,
            String,
            String,
            watch::Receiver<Option<bytes::Bytes>>,
        ),
    ),
    ServiceError,
> {
    let encoding = endpoint.encoding;
    let queryable = declare(backend, key.as_str()).await?;
    let latest = watch::Sender::new(None);
    let pending = (queryable, key.clone(), encoding.clone(), latest.subscribe());
    let published = PublishedState {
        key,
        endpoint: StateEndpoint {
            name: endpoint.name,
            encoding,
            project: endpoint.project,
        },
        latest,
    };
    Ok((published, pending))
}
