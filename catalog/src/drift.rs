use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::observed::ObservedFacts;
use crate::service::ServiceDefinition;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DriftReport {
    pub findings: Vec<DriftFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DriftFinding {
    pub field: String,
    pub message: String,
}

impl DriftReport {
    pub fn no_drift() -> Self {
        Self {
            findings: Vec::new(),
        }
    }

    pub fn has_drift(&self) -> bool {
        !self.findings.is_empty()
    }
}

pub fn diff(asserted: &ServiceDefinition, observed: &ObservedFacts) -> DriftReport {
    let _ = (asserted, observed);
    DriftReport::no_drift()
}

pub fn diff_catalog(services: &[ServiceDefinition], observed: &[ObservedFacts]) -> DriftReport {
    let mut report = DriftReport::no_drift();
    for service in services {
        if let Some(facts) = observed.iter().find(|f| f.id == service.id) {
            let service_report = diff(service, facts);
            report.findings.extend(service_report.findings);
        }
    }
    report
}
