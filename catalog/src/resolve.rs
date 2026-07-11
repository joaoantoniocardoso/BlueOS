use thiserror::Error;

use crate::catalog::Catalog;
use crate::id::{Port, PortRef, ServiceId};
use crate::observed::ObservedFacts;

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
    pub ports: Vec<Port>,
}

pub fn resolve(catalog: &Catalog, env: &EnvLookup) -> Result<Vec<ResolvedPorts>, ResolveError> {
    catalog
        .observed()
        .iter()
        .map(|facts| {
            Ok(ResolvedPorts {
                service_id: facts.id.clone(),
                ports: resolve_service_ports(facts, env)?,
            })
        })
        .collect()
}

pub fn resolve_port_ref(port_ref: &PortRef, env: &EnvLookup) -> Result<Port, ResolveError> {
    match port_ref {
        PortRef::Literal(port) => Ok(Port(*port)),
        PortRef::Env(name) => {
            let value = env(name).ok_or_else(|| ResolveError::EnvNotSet(name.clone()))?;
            let port = value
                .parse::<u16>()
                .map_err(|_| ResolveError::InvalidPort(name.clone(), value))?;
            Ok(Port(port))
        }
    }
}

pub fn resolve_service_ports(
    facts: &ObservedFacts,
    env: &EnvLookup,
) -> Result<Vec<Port>, ResolveError> {
    match &facts.listen {
        crate::provenance::ObservedSet::Known { items } => items
            .iter()
            .map(|evidenced| resolve_port_ref(&evidenced.value, env))
            .collect(),
        crate::provenance::ObservedSet::Unknown { .. } => Ok(Vec::new()),
    }
}
