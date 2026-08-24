use schemars::JsonSchema;
use serde::Serialize;
use std::collections::HashSet;

use crate::observed::ObservedFacts;
use crate::provenance::{AssertedSet, GroundedSet, ObservedSet};
use crate::runtime::RuntimeFacts;
use crate::service::{Service, ServiceJudgment};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct DriftReport {
    pub findings: Vec<DriftFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
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

pub fn diff(asserted: &ServiceJudgment, observed: &ObservedFacts) -> DriftReport {
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
            .map(|item| item.value.path.0)
            .collect();
        for resource in asserted_resources.iter() {
            let path = &resource.value.path;
            if !observed_paths.contains(path.0) {
                report.findings.push(DriftFinding {
                    field: format!("{}.resources", asserted.id.as_str()),
                    message: format!("asserted resource '{}' has no observed evidence", path.0),
                });
            }
        }
    }
    report
}

pub fn diff_runtime(asserted: &ServiceJudgment, runtime: &RuntimeFacts) -> DriftReport {
    let mut report = DriftReport::no_drift();

    // Status codes are not a sound falsifier of api_stable, which is about versioned API surface;
    // contractual 5xx are captured as StateContracts.

    if let GroundedSet::Known { items } = &runtime.state_contracts {
        let mut seen_missing_machines = HashSet::new();
        let mut seen_missing_states = HashSet::new();

        for item in items.iter() {
            let machine = &item.value.machine;
            let state = &item.value.state;

            match &asserted.states {
                AssertedSet::Unknown { .. } => {
                    if seen_missing_machines.insert(*machine) {
                        report.findings.push(DriftFinding {
                            field: format!("{}.states", asserted.id.as_str()),
                            message: format!(
                                "runtime references state machine '{}' but asserted states is Unknown",
                                machine
                            ),
                        });
                    }
                }
                AssertedSet::Established { items: sms } => {
                    let Some(sm) = sms.iter().find(|sm| sm.value.name == *machine) else {
                        if seen_missing_machines.insert(*machine) {
                            report.findings.push(DriftFinding {
                                field: format!("{}.states", asserted.id.as_str()),
                                message: format!(
                                    "runtime references state machine '{}' not present in asserted states",
                                    machine
                                ),
                            });
                        }
                        continue;
                    };
                    if !sm.value.states.iter().any(|s| s == state) {
                        let key = (*machine, *state);
                        if seen_missing_states.insert(key) {
                            report.findings.push(DriftFinding {
                                field: format!("{}.states", asserted.id.as_str()),
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

pub fn diff_catalog(services: &[Service]) -> DriftReport {
    let mut report = DriftReport::no_drift();
    for service in services {
        let service_report = diff(&service.definition, &service.observed);
        report.findings.extend(service_report.findings);
    }
    report
}

pub fn diff_catalog_runtime(services: &[Service]) -> DriftReport {
    let mut report = DriftReport::no_drift();
    for service in services {
        let service_report = diff_runtime(&service.definition, &service.runtime);
        report.findings.extend(service_report.findings);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::id::{PathRef, ServiceId};
    use crate::journey::{HttpMethod, RouteRef};
    use crate::provenance::{AssertedSet, GroundedItem, GroundedSet, Provenance, Rationaled};
    use crate::resource::{Resource, ResourceOwnership};
    use crate::runtime::{RuntimeFacts, StateContract};

    #[test]
    fn bootstrap_has_no_asserted_observed_drift() {
        let catalog = Catalog::bootstrap();
        let report = diff_catalog(catalog.services());
        assert!(!report.has_drift(), "{:?}", report.findings);
    }

    #[test]
    fn bootstrap_has_no_runtime_drift() {
        let catalog = Catalog::bootstrap();
        let report = diff_catalog_runtime(catalog.services());
        assert!(!report.has_drift(), "{:?}", report.findings);
    }

    #[test]
    fn detects_missing_observed_resource() {
        let catalog = Catalog::bootstrap();
        let mut service = catalog.services()[0].definition.clone();
        service.resources = AssertedSet::established(
            const {
                &[Rationaled::new(
                    Resource {
                        path: PathRef("/bogus/missing/path"),
                        ownership: ResourceOwnership::Exclusive,
                    },
                    "test",
                )]
            },
        );
        let observed = catalog.services()[0].observed.clone();
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
            .find(|s| s.id.as_str() == "ardupilot_manager")
            .expect("ardupilot_manager in bootstrap")
            .definition
            .clone();
        let runtime = RuntimeFacts {
            service: service.id,
            state_contracts: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        StateContract {
                            machine: "autopilot_lifecycle",
                            state: "bogus_state",
                            route: RouteRef {
                                service: ServiceId::ArdupilotManager,
                                method: HttpMethod::Get,
                                path: "/vehicle_type",
                                version: None,
                            },
                            status: 200,
                            body_predicate: None,
                        },
                        Provenance::asserted("test"),
                    )]
                },
            ),
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
