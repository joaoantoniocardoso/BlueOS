use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::ServiceId;
use crate::observed::ObservedFacts;
use crate::service::ServiceDefinition;
use crate::validate::ValidationError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Catalog {
    services: Vec<ServiceDefinition>,
    observed: Vec<ObservedFacts>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CouplingMatrix {
    pub service_ids: Vec<ServiceId>,
    pub weights: Vec<Vec<f64>>,
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            observed: Vec::new(),
        }
    }

    pub fn with_parts(services: Vec<ServiceDefinition>, observed: Vec<ObservedFacts>) -> Self {
        Self { services, observed }
    }

    pub fn services(&self) -> &[ServiceDefinition] {
        &self.services
    }

    pub fn observed(&self) -> &[ObservedFacts] {
        &self.observed
    }

    pub fn service_by_id(&self, id: &ServiceId) -> Option<&ServiceDefinition> {
        self.services.iter().find(|service| &service.id == id)
    }

    pub fn observed_by_id(&self, id: &ServiceId) -> Option<&ObservedFacts> {
        self.observed.iter().find(|facts| &facts.id == id)
    }

    pub fn service_index(&self) -> HashMap<&ServiceId, &ServiceDefinition> {
        self.services
            .iter()
            .map(|service| (&service.id, service))
            .collect()
    }

    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        crate::validate::validate(self)
    }

    pub fn coupling_matrix(&self) -> CouplingMatrix {
        let service_ids: Vec<ServiceId> = self.services.iter().map(|s| s.id.clone()).collect();
        let size = service_ids.len();
        CouplingMatrix {
            service_ids,
            weights: vec![vec![0.0; size]; size],
        }
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}
