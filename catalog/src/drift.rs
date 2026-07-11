use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::observed::ObservedFacts;
use crate::provenance::{AssertedSet, GroundedSet, ObservedSet};
use crate::runtime::RuntimeFacts;
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
    let mut report = DriftReport::no_drift();
    if let (
        AssertedSet::Established {
            items: asserted_resources,
        },
        ObservedSet::Known {
            items: observed_resources,
        },
    ) = (&asserted.resources, &observed.resources)
    {
        // Ownership is a judgment layer; drift only checks that asserted paths have extraction evidence.
        let observed_paths: HashSet<&str> = observed_resources
            .iter()
            .map(|item| item.value.path.0.as_str())
            .collect();
        for resource in asserted_resources {
            let path = &resource.value.path;
            if !observed_paths.contains(path.0.as_str()) {
                report.findings.push(DriftFinding {
                    field: format!("{}.resources", asserted.id.0),
                    message: format!("asserted resource '{}' has no observed evidence", path.0),
                });
            }
        }
    }
    report
}

pub fn diff_runtime(asserted: &ServiceDefinition, runtime: &RuntimeFacts) -> DriftReport {
    let mut report = DriftReport::no_drift();

    // Status codes are not a sound falsifier of api_stable, which is about versioned API surface;
    // contractual 5xx are captured as StateContracts.

    if let GroundedSet::Known { items } = &runtime.state_contracts {
        let mut seen_missing_machines = HashSet::new();
        let mut seen_missing_states = HashSet::new();

        for item in items {
            let machine = &item.value.machine;
            let state = &item.value.state;

            match &asserted.states {
                AssertedSet::Unknown { .. } => {
                    if seen_missing_machines.insert(machine.clone()) {
                        report.findings.push(DriftFinding {
                            field: format!("{}.states", asserted.id.0),
                            message: format!(
                                "runtime references state machine '{}' but asserted states is Unknown",
                                machine
                            ),
                        });
                    }
                }
                AssertedSet::Established { items: sms } => {
                    let Some(sm) = sms.iter().find(|sm| sm.value.name == *machine) else {
                        if seen_missing_machines.insert(machine.clone()) {
                            report.findings.push(DriftFinding {
                                field: format!("{}.states", asserted.id.0),
                                message: format!(
                                    "runtime references state machine '{}' not present in asserted states",
                                    machine
                                ),
                            });
                        }
                        continue;
                    };
                    if !sm.value.states.iter().any(|s| s == state) {
                        let key = (machine.clone(), state.clone());
                        if seen_missing_states.insert(key) {
                            report.findings.push(DriftFinding {
                                field: format!("{}.states", asserted.id.0),
                                message: format!(
                                    "runtime references state '{}' not declared in asserted state machine '{}'",
                                    state, machine
                                ),
                            });
                        }
                    }
                }
            }
        }
    }

    report
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

pub fn diff_catalog_runtime(
    services: &[ServiceDefinition],
    runtime: &[RuntimeFacts],
) -> DriftReport {
    let mut report = DriftReport::no_drift();
    for service in services {
        if let Some(facts) = runtime.iter().find(|f| f.service == service.id) {
            let service_report = diff_runtime(service, facts);
            report.findings.extend(service_report.findings);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::id::PathRef;
    use crate::journey::{HttpMethod, RouteRef};
    use crate::provenance::{AssertedSet, GroundedItem, GroundedSet, Provenance, Rationaled};
    use crate::resource::{Resource, ResourceOwnership};
    use crate::runtime::{RuntimeFacts, StateContract};

    #[test]
    fn bootstrap_has_no_asserted_observed_drift() {
        let catalog = Catalog::bootstrap();
        let report = diff_catalog(catalog.services(), catalog.observed());
        assert!(!report.has_drift(), "{:?}", report.findings);
    }

    #[test]
    fn bootstrap_has_no_runtime_drift() {
        let catalog = Catalog::bootstrap();
        let report = diff_catalog_runtime(catalog.services(), catalog.runtime());
        assert!(!report.has_drift(), "{:?}", report.findings);
    }

    #[test]
    fn detects_missing_observed_resource() {
        let catalog = Catalog::bootstrap();
        let mut service = catalog.services()[0].clone();
        service.resources = AssertedSet::established(vec![Rationaled::new(
            Resource {
                path: PathRef("/bogus/missing/path".to_string()),
                ownership: ResourceOwnership::Exclusive,
            },
            "test",
        )]);
        let observed = catalog.observed()[0].clone();
        let report = diff(&service, &observed);
        assert!(report.has_drift());
        assert!(report
            .findings
            .iter()
            .any(|f| f.message.contains("/bogus/missing/path")));
    }

    #[test]
    fn detects_undeclared_runtime_state() {
        let catalog = Catalog::bootstrap();
        let service = catalog
            .services()
            .iter()
            .find(|s| s.id.0 == "ardupilot_manager")
            .expect("ardupilot_manager in bootstrap")
            .clone();
        let runtime = RuntimeFacts {
            service: service.id.clone(),
            state_contracts: GroundedSet::known(vec![GroundedItem::new(
                StateContract {
                    machine: "autopilot_lifecycle".to_string(),
                    state: "bogus_state".to_string(),
                    route: RouteRef {
                        service: service.id.clone(),
                        method: HttpMethod::Get,
                        path: "/vehicle_type".to_string(),
                        version: None,
                    },
                    status: 200,
                    body_predicate: None,
                },
                Provenance::asserted("test"),
            )]),
            slo_baselines: GroundedSet::unknown("test"),
            resource_usage: GroundedSet::unknown("test"),
            platform_matrix: GroundedSet::unknown("test"),
            settings_mutations: GroundedSet::unknown("test"),
        };
        let report = diff_runtime(&service, &runtime);
        assert!(report.has_drift());
        assert!(report.findings.iter().any(|f| {
            f.message.contains("bogus_state") && f.message.contains("autopilot_lifecycle")
        }));
    }
}
