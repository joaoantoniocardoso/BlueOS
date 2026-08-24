use thiserror::Error;

use catalog_kernel::id::refs::{PortRef, TcpPort};
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::ObservedSet;
use catalog_model::observed::ObservedFacts;

use crate::catalog::Catalog;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ResolveError {
    #[error("environment variable {0} is not set")]
    EnvNotSet(String),
    #[error("environment variable {0} is not a valid port: {1}")]
    InvalidPort(String, String),
}

pub type EnvLookup<'a> = dyn Fn(&str) -> Option<String> + 'a;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPorts {
    pub service_id: ServiceId,
    pub ports: Vec<TcpPort>,
}

pub fn resolve(catalog: &Catalog, env: &EnvLookup) -> Result<Vec<ResolvedPorts>, ResolveError> {
    catalog
        .services()
        .iter()
        .map(|service| {
            Ok(ResolvedPorts {
                service_id: service.id,
                ports: resolve_service_ports(&service.observed, env)?,
            })
        })
        .collect()
}

pub fn resolve_port_ref(port_ref: &PortRef, env: &EnvLookup) -> Result<TcpPort, ResolveError> {
    match port_ref {
        PortRef::Literal(port) => Ok(TcpPort(*port)),
        PortRef::Env(name) => {
            let value = env(name).ok_or_else(|| ResolveError::EnvNotSet(name.to_string()))?;
            let port = value
                .parse::<u16>()
                .map_err(|_| ResolveError::InvalidPort(name.to_string(), value))?;
            Ok(TcpPort(port))
        }
    }
}

pub fn resolve_service_ports(
    facts: &ObservedFacts,
    env: &EnvLookup,
) -> Result<Vec<TcpPort>, ResolveError> {
    match &facts.listen {
        ObservedSet::Known { items } => items
            .iter()
            .map(|evidenced| resolve_port_ref(&evidenced.value, env))
            .collect(),
        ObservedSet::Unknown { .. } => Ok(Vec::new()),
    }
}
