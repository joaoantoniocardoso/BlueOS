use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::catalog::Catalog;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::UserJourney;
use crate::provenance::{AssertedSet, GroundedSet, ObservedSet};
use crate::resource::ResourceOwnership;
use crate::service::{Authority, ServiceDefinition};
use crate::state::StateMachine;

// M1 will calibrate this against reference service cards.
pub const COVERAGE_UNKNOWN_THRESHOLD: usize = 10_000;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("exclusive resource {resource} claimed by {first} and {second}")]
    DuplicateExclusiveResource {
        resource: String,
        first: String,
        second: String,
    },
    #[error("authority {authority} claimed by {first} and {second}")]
    ConflictingAuthority {
        authority: String,
        first: String,
        second: String,
    },
    #[error("edge from {from} targets unknown service {to}")]
    UnknownEdgeTarget { from: String, to: String },
    #[error("coverage gate failed: {unknown_count} unknown required fields exceeds threshold {threshold}")]
    CoverageGateFailed {
        unknown_count: usize,
        threshold: usize,
    },
    #[error("journey {journey} references unknown service {service}")]
    UnknownJourneyService { journey: String, service: String },
    #[error("journey {journey} references unknown capability {capability}")]
    UnknownJourneyCapability { journey: String, capability: String },
    #[error("journey {journey} references unknown state {state} in machine {machine}")]
    UnknownJourneyState {
        journey: String,
        machine: String,
        state: String,
    },
    #[error("journey {journey} chains from unknown journey {chains_from}")]
    UnknownJourneyChain {
        journey: String,
        chains_from: String,
    },
    #[error("service {service} references unknown journey {journey}")]
    UnknownServiceJourneyRef { service: String, journey: String },
    #[error("runtime facts reference unknown service {service}")]
    UnknownRuntimeService { service: String },
    #[error(
        "runtime facts reference unknown state {state} in machine {machine} for service {service}"
    )]
    UnknownRuntimeState {
        service: String,
        machine: String,
        state: String,
    },
}

pub fn validate(catalog: &Catalog) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    errors.extend(check_exclusive_resources(catalog));
    errors.extend(check_conflicting_authorities(catalog));
    errors.extend(check_edge_targets(catalog));
    errors.extend(check_service_journey_refs(catalog));
    errors.extend(check_journey_references(catalog));
    check_runtime_references(catalog, &mut errors);
    errors.extend(check_coverage_gate(catalog));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_exclusive_resources(catalog: &Catalog) -> Vec<ValidationError> {
    let mut claims: HashMap<String, ServiceId> = HashMap::new();
    let mut errors = Vec::new();

    for service in catalog.services() {
        if let Some(observed) = catalog.observed_by_id(&service.id) {
            if let ObservedSet::Known { items } = &observed.listen {
                for port_ref in items.iter().map(|e| &e.value) {
                    if let crate::id::PortRef::Literal(port) = port_ref {
                        let key = format!("port:{port}");
                        track_exclusive_claim(&mut claims, &mut errors, &service.id, key);
                    }
                }
            }
        }

        if let AssertedSet::Established { items } = &service.resources {
            for resource in items.iter().map(|r| &r.value) {
                if resource.ownership == ResourceOwnership::Exclusive {
                    let key = format!("resource:{}", resource.path.0);
                    track_exclusive_claim(&mut claims, &mut errors, &service.id, key);
                }
            }
        }
    }

    errors
}

fn track_exclusive_claim(
    claims: &mut HashMap<String, ServiceId>,
    errors: &mut Vec<ValidationError>,
    service_id: &ServiceId,
    key: String,
) {
    if let Some(existing) = claims.get(&key) {
        if existing != service_id {
            errors.push(ValidationError::DuplicateExclusiveResource {
                resource: key,
                first: existing.0.clone(),
                second: service_id.0.clone(),
            });
        }
    } else {
        claims.insert(key, service_id.clone());
    }
}

fn check_conflicting_authorities(catalog: &Catalog) -> Vec<ValidationError> {
    let mut claims: HashMap<String, ServiceId> = HashMap::new();
    let mut errors = Vec::new();

    for service in catalog.services() {
        if let AssertedSet::Established { items } = &service.authorities {
            for authority in items.iter().map(|a| &a.value) {
                let key = authority_key(authority);
                if let Some(existing) = claims.get(&key) {
                    if existing != &service.id {
                        errors.push(ValidationError::ConflictingAuthority {
                            authority: key,
                            first: existing.0.clone(),
                            second: service.id.0.clone(),
                        });
                    }
                } else {
                    claims.insert(key, service.id.clone());
                }
            }
        }
    }

    errors
}

fn authority_key(authority: &Authority) -> String {
    match authority {
        Authority::MavlinkRouterOwner => "mavlink_router_owner".to_string(),
        Authority::NginxProxy => "nginx_proxy".to_string(),
        Authority::ZenohBroker => "zenoh_broker".to_string(),
        Authority::UserdataWriter(path) => format!("userdata_writer:{}", path.0),
        Authority::HardwareExclusive(path) => format!("hardware_exclusive:{}", path.0),
        Authority::Other(name) => format!("other:{name}"),
    }
}

fn check_edge_targets(catalog: &Catalog) -> Vec<ValidationError> {
    let known_ids: HashSet<&ServiceId> = catalog.services().iter().map(|s| &s.id).collect();
    let mut errors = Vec::new();

    for service in catalog.services() {
        if let AssertedSet::Established { items } = &service.edges {
            for edge in items.iter().map(|e| &e.value) {
                if !known_ids.contains(&edge.to) {
                    errors.push(ValidationError::UnknownEdgeTarget {
                        from: service.id.0.clone(),
                        to: edge.to.0.clone(),
                    });
                }
            }
        }
    }

    errors
}

fn check_service_journey_refs(catalog: &Catalog) -> Vec<ValidationError> {
    let known_journey_ids: HashSet<&JourneyId> = catalog.journeys().iter().map(|j| &j.id).collect();
    let mut errors = Vec::new();

    for service in catalog.services() {
        if let AssertedSet::Established { items } = &service.journey_refs {
            for journey_ref in items.iter().map(|j| &j.value) {
                if !known_journey_ids.contains(journey_ref) {
                    errors.push(ValidationError::UnknownServiceJourneyRef {
                        service: service.id.0.clone(),
                        journey: journey_ref.0.clone(),
                    });
                }
            }
        }
    }

    errors
}

fn check_journey_references(catalog: &Catalog) -> Vec<ValidationError> {
    let known_service_ids: HashSet<&ServiceId> = catalog.services().iter().map(|s| &s.id).collect();
    let known_journey_ids: HashSet<&JourneyId> = catalog.journeys().iter().map(|j| &j.id).collect();
    let service_index = catalog.service_index();
    let mut errors = Vec::new();

    for journey in catalog.journeys() {
        let journey_id = journey.id.0.clone();
        let participating = participating_service_ids(journey);

        if let Some(chains_from) = &journey.chains_from {
            if !known_journey_ids.contains(chains_from) {
                errors.push(ValidationError::UnknownJourneyChain {
                    journey: journey_id.clone(),
                    chains_from: chains_from.0.clone(),
                });
            }
        }

        if let GroundedSet::Known { items } = &journey.services {
            for item in items {
                if !known_service_ids.contains(&item.value) {
                    errors.push(ValidationError::UnknownJourneyService {
                        journey: journey_id.clone(),
                        service: item.value.0.clone(),
                    });
                }
            }
        }

        if let GroundedSet::Known { items } = &journey.capability_refs {
            for item in items {
                if !capability_in_participating_services(
                    &item.value,
                    &participating,
                    &service_index,
                ) {
                    errors.push(ValidationError::UnknownJourneyCapability {
                        journey: journey_id.clone(),
                        capability: item.value.0.clone(),
                    });
                }
            }
        }

        if let GroundedSet::Known { items } = &journey.steps {
            for item in items {
                if let Some(crate::provenance::Grounded::Known { value: route, .. }) =
                    &item.value.route
                {
                    if !known_service_ids.contains(&route.service) {
                        errors.push(ValidationError::UnknownJourneyService {
                            journey: journey_id.clone(),
                            service: route.service.0.clone(),
                        });
                    }
                }
                if let Some(crate::provenance::Grounded::Known { value: outcome, .. }) =
                    &item.value.outcome
                {
                    if let Some(transition) = &outcome.transition {
                        for state in [&transition.from, &transition.to] {
                            if !state_in_participating_services(
                                &transition.machine,
                                state,
                                &participating,
                                &service_index,
                            ) {
                                errors.push(ValidationError::UnknownJourneyState {
                                    journey: journey_id.clone(),
                                    machine: transition.machine.clone(),
                                    state: state.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    errors
}

fn participating_service_ids(journey: &UserJourney) -> Vec<&ServiceId> {
    match &journey.services {
        GroundedSet::Known { items } => items.iter().map(|item| &item.value).collect(),
        GroundedSet::Unknown { .. } => Vec::new(),
    }
}

fn capability_in_participating_services(
    capability: &CapabilityId,
    participating: &[&ServiceId],
    service_index: &HashMap<&ServiceId, &ServiceDefinition>,
) -> bool {
    participating.iter().any(|service_id| {
        service_index
            .get(service_id)
            .and_then(|service| match &service.capabilities {
                AssertedSet::Established { items } => {
                    Some(items.iter().any(|item| &item.value == capability))
                }
                AssertedSet::Unknown { .. } => None,
            })
            .unwrap_or(false)
    })
}

fn state_in_participating_services(
    machine: &str,
    state: &str,
    participating: &[&ServiceId],
    service_index: &HashMap<&ServiceId, &ServiceDefinition>,
) -> bool {
    participating.iter().any(|service_id| {
        service_index
            .get(service_id)
            .and_then(|service| match &service.states {
                AssertedSet::Established { items } => {
                    Some(state_in_machines(machine, state, items))
                }
                AssertedSet::Unknown { .. } => None,
            })
            .unwrap_or(false)
    })
}

fn state_in_machines(
    machine: &str,
    state: &str,
    machines: &[crate::provenance::Rationaled<StateMachine>],
) -> bool {
    machines
        .iter()
        .any(|item| item.value.name == machine && item.value.states.iter().any(|s| s == state))
}

fn check_runtime_references(catalog: &Catalog, errors: &mut Vec<ValidationError>) {
    let known_service_ids: HashSet<&ServiceId> = catalog.services().iter().map(|s| &s.id).collect();
    let service_index = catalog.service_index();

    for facts in catalog.runtime() {
        let service_id = facts.service.0.clone();
        if !known_service_ids.contains(&facts.service) {
            errors.push(ValidationError::UnknownRuntimeService {
                service: service_id.clone(),
            });
            continue;
        }

        if let GroundedSet::Known { items } = &facts.state_contracts {
            for item in items {
                let contract = &item.value;
                if !state_in_service(
                    &facts.service,
                    &contract.machine,
                    &contract.state,
                    &service_index,
                ) {
                    errors.push(ValidationError::UnknownRuntimeState {
                        service: service_id.clone(),
                        machine: contract.machine.clone(),
                        state: contract.state.clone(),
                    });
                }
            }
        }

        // Route refs in state_contracts and slo_baselines are not cross-checked until a route inventory exists.
        // resource_usage carries no catalog references and needs no cross-ref check.
    }
}

fn state_in_service(
    service_id: &ServiceId,
    machine: &str,
    state: &str,
    service_index: &HashMap<&ServiceId, &ServiceDefinition>,
) -> bool {
    service_index
        .get(service_id)
        .and_then(|service| match &service.states {
            AssertedSet::Established { items } => Some(state_in_machines(machine, state, items)),
            AssertedSet::Unknown { .. } => None,
        })
        .unwrap_or(false)
}

fn check_coverage_gate(catalog: &Catalog) -> Vec<ValidationError> {
    let unknown_count: usize = catalog
        .services()
        .iter()
        .map(count_unknown_in_service)
        .sum();

    if unknown_count > COVERAGE_UNKNOWN_THRESHOLD {
        vec![ValidationError::CoverageGateFailed {
            unknown_count,
            threshold: COVERAGE_UNKNOWN_THRESHOLD,
        }]
    } else {
        Vec::new()
    }
}

fn count_unknown_in_service(service: &ServiceDefinition) -> usize {
    let mut count = 0;
    count += service.singleton.is_unknown() as usize;
    count += service.bounded_context.is_unknown() as usize;
    count += service.journey_refs.is_unknown() as usize;
    count += service.tier.is_unknown() as usize;
    count += service.offline_required.is_unknown() as usize;
    count += service.privilege_level.is_unknown() as usize;
    count += service.dangerous_operations.is_unknown() as usize;
    count += service.user_confirmation.is_unknown() as usize;
    count += service.capabilities.is_unknown() as usize;
    count += service.authorities.is_unknown() as usize;
    count += service.states.is_unknown() as usize;
    count += service.edges.is_unknown() as usize;
    count += service.resources.is_unknown() as usize;
    count += service.lifecycle.triggers.is_unknown() as usize;
    count += service.lifecycle.ordered_after.is_unknown() as usize;
    count += service.lifecycle.ordered_before.is_unknown() as usize;
    count += service.lifecycle.shutdown.is_unknown() as usize;
    count += service.lifecycle.upgrade_behavior.is_unknown() as usize;
    count += service.health.is_unknown() as usize;
    count += service.is_platform.is_unknown() as usize;
    count += service.api_stable.is_unknown() as usize;
    count += service.permissions_model.is_unknown() as usize;
    count += service.failure_modes.is_unknown() as usize;
    count += service.blast_radius.is_unknown() as usize;
    count += service.compatibility_policy.is_unknown() as usize;
    count += service.team.is_unknown() as usize;
    count += service.adr_refs.is_unknown() as usize;
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::{Bus, Edge, FailureImpact, SyncMode};
    use crate::id::{CapabilityId, JourneyId, PathRef, ServiceId};
    use crate::journey::{
        Actor, HttpMethod, JourneyStep, RouteRef, StateTransition, StepOutcome, UserJourney,
        Visibility,
    };
    use crate::lifecycle::Lifecycle;
    use crate::observed::ObservedFacts;
    use crate::provenance::{
        Asserted, AssertedSet, Evidence, Evidenced, Grounded, GroundedItem, GroundedSet, Observed,
        ObservedSet, Provenance, Rationaled,
    };
    use crate::resource::{Resource, ResourceOwnership};
    use crate::runtime::{RuntimeFacts, StateContract};
    use crate::state::StateMachine;

    fn evidence() -> Evidence {
        Evidence {
            file: "test.rs".to_string(),
            line: 1,
        }
    }

    fn empty_service(id: &str) -> ServiceDefinition {
        ServiceDefinition {
            id: ServiceId(id.to_string()),
            singleton: Asserted::unknown("not established"),
            bounded_context: Asserted::unknown("not established"),
            journey_refs: AssertedSet::unknown("not established"),
            tier: Asserted::unknown("not established"),
            offline_required: Asserted::unknown("not established"),
            privilege_level: Asserted::unknown("not established"),
            dangerous_operations: AssertedSet::unknown("not established"),
            user_confirmation: Asserted::unknown("not established"),
            capabilities: AssertedSet::unknown("not established"),
            authorities: AssertedSet::unknown("not established"),
            states: AssertedSet::unknown("not established"),
            edges: AssertedSet::unknown("not established"),
            resources: AssertedSet::unknown("not established"),
            lifecycle: Lifecycle {
                triggers: Asserted::unknown("not established"),
                ordered_after: Asserted::unknown("not established"),
                ordered_before: Asserted::unknown("not established"),
                shutdown: Asserted::unknown("not established"),
                upgrade_behavior: Asserted::unknown("not established"),
            },
            health: Asserted::unknown("not established"),
            is_platform: Asserted::unknown("not established"),
            api_stable: Asserted::unknown("not established"),
            permissions_model: Asserted::unknown("not established"),
            failure_modes: AssertedSet::unknown("not established"),
            blast_radius: Asserted::unknown("not established"),
            compatibility_policy: Asserted::unknown("not established"),
            team: Asserted::unknown("not established"),
            adr_refs: AssertedSet::unknown("not established"),
        }
    }

    fn empty_observed(id: &str) -> ObservedFacts {
        ObservedFacts {
            id: ServiceId(id.to_string()),
            aliases: ObservedSet::unknown("not extracted"),
            kind: Observed::unknown("not extracted"),
            entrypoint: Observed::unknown("not extracted"),
            tmux_name: Observed::unknown("not extracted"),
            startup_tier: Observed::unknown("not extracted"),
            resource_limits: Observed::unknown("not extracted"),
            nice: Observed::unknown("not extracted"),
            run_as: Observed::unknown("not extracted"),
            nginx_prefixes: ObservedSet::unknown("not extracted"),
            listen: ObservedSet::unknown("not extracted"),
            git_path: Observed::unknown("not extracted"),
            interfaces: ObservedSet::unknown("not extracted"),
            resources: ObservedSet::unknown("not extracted"),
            lifecycle: Observed::unknown("not extracted"),
            logs_path: Observed::unknown("not extracted"),
            zenoh_log_topic: Observed::unknown("not extracted"),
            sentry: Observed::unknown("not extracted"),
            openapi_refs: ObservedSet::unknown("not extracted"),
        }
    }

    #[test]
    fn empty_catalog_passes_validate() {
        let catalog = Catalog::new();
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn bootstrap_catalog_passes_validate() {
        let catalog = Catalog::bootstrap();
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn minimal_catalog_with_unknown_fields_passes() {
        let service = empty_service("helper");
        let observed = empty_observed("helper");
        let catalog = Catalog::with_parts(vec![service], vec![observed], vec![], vec![]);
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn duplicate_exclusive_port_fails() {
        let mut service_a = empty_service("a");
        service_a.resources = AssertedSet::established(vec![Rationaled::new(
            Resource {
                path: PathRef("/dev/ttyUSB0".to_string()),
                ownership: ResourceOwnership::Exclusive,
            },
            "test",
        )]);
        let mut service_b = empty_service("b");
        service_b.resources = AssertedSet::established(vec![Rationaled::new(
            Resource {
                path: PathRef("/dev/ttyUSB0".to_string()),
                ownership: ResourceOwnership::Exclusive,
            },
            "test",
        )]);

        let mut observed_a = empty_observed("a");
        observed_a.listen = ObservedSet::known(vec![Evidenced::new(
            crate::id::PortRef::Literal(8000),
            evidence(),
        )]);
        let mut observed_b = empty_observed("b");
        observed_b.listen = ObservedSet::known(vec![Evidenced::new(
            crate::id::PortRef::Literal(8000),
            evidence(),
        )]);

        let catalog = Catalog::with_parts(
            vec![service_a, service_b],
            vec![observed_a, observed_b],
            vec![],
            vec![],
        );
        let result = catalog.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateExclusiveResource { .. })));
    }

    #[test]
    fn unknown_edge_target_fails() {
        let mut service_a = empty_service("a");
        service_a.edges = AssertedSet::established(vec![Rationaled::new(
            Edge {
                from: ServiceId("a".to_string()),
                to: ServiceId("missing".to_string()),
                via: Bus::Rest,
                sync: SyncMode::Sync,
                endpoint: "/helper/".to_string(),
                purpose: "call helper".to_string(),
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            },
            "test",
        )]);

        let catalog =
            Catalog::with_parts(vec![service_a], vec![empty_observed("a")], vec![], vec![]);
        let result = catalog.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownEdgeTarget { .. })));
    }

    fn valid_journey_service(id: &str) -> ServiceDefinition {
        let mut service = empty_service(id);
        service.capabilities = AssertedSet::established(vec![Rationaled::new(
            CapabilityId("deploy".to_string()),
            "test",
        )]);
        service.states = AssertedSet::established(vec![Rationaled::new(
            StateMachine {
                name: "lifecycle".to_string(),
                states: vec!["idle".to_string(), "running".to_string()],
                boot_state: "idle".to_string(),
                degraded_when: vec![],
            },
            "test",
        )]);
        service
    }

    fn valid_journey() -> UserJourney {
        UserJourney {
            id: JourneyId("deploy".to_string()),
            summary: Grounded::known(
                "deploy vehicle".to_string(),
                Provenance::doc("docs/deploy.md", 1),
            ),
            visibility: Grounded::known(Visibility::Default, Provenance::doc("docs/deploy.md", 2)),
            services: GroundedSet::known(vec![GroundedItem::new(
                ServiceId("helper".to_string()),
                Provenance::doc("docs/deploy.md", 3),
            )]),
            capability_refs: GroundedSet::known(vec![GroundedItem::new(
                CapabilityId("deploy".to_string()),
                Provenance::doc("docs/deploy.md", 4),
            )]),
            preconditions: GroundedSet::unknown("not grounded"),
            steps: GroundedSet::known(vec![GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "call deploy endpoint".to_string(),
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId("helper".to_string()),
                            method: HttpMethod::Post,
                            path: "/deploy".to_string(),
                            version: Some("v1".to_string()),
                        },
                        Provenance::doc("docs/deploy.md", 5),
                    )),
                    outcome: Some(Grounded::known(
                        StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: Some(StateTransition {
                                machine: "lifecycle".to_string(),
                                from: "idle".to_string(),
                                to: "running".to_string(),
                            }),
                        },
                        Provenance::runtime("tests/baselines/helper.json", "lab"),
                    )),
                },
                Provenance::source("helper/main.py", 10),
            )]),
            chains_from: None,
        }
    }

    #[test]
    fn valid_journey_passes_validate() {
        let service = valid_journey_service("helper");
        let observed = empty_observed("helper");
        let catalog =
            Catalog::with_parts(vec![service], vec![observed], vec![valid_journey()], vec![]);
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn unknown_journey_service_ref_fails() {
        let service = valid_journey_service("helper");
        let journey = UserJourney {
            services: GroundedSet::known(vec![GroundedItem::new(
                ServiceId("missing".to_string()),
                Provenance::doc("docs/deploy.md", 1),
            )]),
            ..valid_journey()
        };
        let catalog = Catalog::with_parts(
            vec![service],
            vec![empty_observed("helper")],
            vec![journey],
            vec![],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownJourneyService { .. })));
    }

    #[test]
    fn unknown_journey_route_service_fails() {
        let service = valid_journey_service("helper");
        let mut journey = valid_journey();
        if let GroundedSet::Known { items } = &mut journey.steps {
            if let Some(Grounded::Known { value: route, .. }) = &mut items[0].value.route {
                route.service = ServiceId("missing".to_string());
            }
        }
        let catalog = Catalog::with_parts(
            vec![service],
            vec![empty_observed("helper")],
            vec![journey],
            vec![],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownJourneyService { .. })));
    }

    #[test]
    fn unknown_journey_capability_fails() {
        let service = valid_journey_service("helper");
        let mut journey = valid_journey();
        journey.capability_refs = GroundedSet::known(vec![GroundedItem::new(
            CapabilityId("missing".to_string()),
            Provenance::doc("docs/deploy.md", 1),
        )]);
        let catalog = Catalog::with_parts(
            vec![service],
            vec![empty_observed("helper")],
            vec![journey],
            vec![],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownJourneyCapability { .. })));
    }

    #[test]
    fn unknown_journey_state_fails() {
        let service = valid_journey_service("helper");
        let mut journey = valid_journey();
        if let GroundedSet::Known { items } = &mut journey.steps {
            if let Some(Grounded::Known { value: outcome, .. }) = &mut items[0].value.outcome {
                outcome.transition = Some(StateTransition {
                    machine: "lifecycle".to_string(),
                    from: "idle".to_string(),
                    to: "missing".to_string(),
                });
            }
        }
        let catalog = Catalog::with_parts(
            vec![service],
            vec![empty_observed("helper")],
            vec![journey],
            vec![],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownJourneyState { .. })));
    }

    #[test]
    fn unknown_journey_state_machine_name_fails() {
        let service = valid_journey_service("helper");
        let mut journey = valid_journey();
        if let GroundedSet::Known { items } = &mut journey.steps {
            if let Some(Grounded::Known { value: outcome, .. }) = &mut items[0].value.outcome {
                outcome.transition = Some(StateTransition {
                    machine: "nonexistent".to_string(),
                    from: "idle".to_string(),
                    to: "running".to_string(),
                });
            }
        }
        let catalog = Catalog::with_parts(
            vec![service],
            vec![empty_observed("helper")],
            vec![journey],
            vec![],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownJourneyState { .. })));
    }

    #[test]
    fn unknown_journey_chain_fails() {
        let service = valid_journey_service("helper");
        let mut journey = valid_journey();
        journey.chains_from = Some(JourneyId("missing".to_string()));
        let catalog = Catalog::with_parts(
            vec![service],
            vec![empty_observed("helper")],
            vec![journey],
            vec![],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownJourneyChain { .. })));
    }

    #[test]
    fn unknown_service_journey_ref_fails() {
        let mut service = empty_service("helper");
        service.journey_refs = AssertedSet::established(vec![Rationaled::new(
            JourneyId("missing".to_string()),
            "test",
        )]);
        let catalog = Catalog::with_parts(
            vec![service],
            vec![empty_observed("helper")],
            vec![],
            vec![],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownServiceJourneyRef { .. })));
    }

    #[test]
    fn unknown_runtime_service_fails() {
        let runtime = RuntimeFacts {
            service: ServiceId("missing".to_string()),
            state_contracts: GroundedSet::unknown("not captured"),
            slo_baselines: GroundedSet::unknown("not captured"),
            resource_usage: GroundedSet::unknown("not captured"),
            platform_matrix: GroundedSet::unknown("not captured"),
            settings_mutations: GroundedSet::unknown("not captured"),
        };
        let catalog =
            Catalog::with_parts(vec![empty_service("helper")], vec![], vec![], vec![runtime]);
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownRuntimeService { .. })));
    }

    #[test]
    fn unknown_runtime_state_fails() {
        let service = valid_journey_service("helper");
        let runtime = RuntimeFacts {
            service: ServiceId("helper".to_string()),
            state_contracts: GroundedSet::known(vec![GroundedItem::new(
                StateContract {
                    machine: "lifecycle".to_string(),
                    state: "missing".to_string(),
                    route: RouteRef {
                        service: ServiceId("helper".to_string()),
                        method: HttpMethod::Get,
                        path: "/status".to_string(),
                        version: None,
                    },
                    status: 200,
                    body_predicate: None,
                },
                Provenance::runtime("runtime-captures/sample.json#k", "lab"),
            )]),
            slo_baselines: GroundedSet::unknown("not captured"),
            resource_usage: GroundedSet::unknown("not captured"),
            platform_matrix: GroundedSet::unknown("not captured"),
            settings_mutations: GroundedSet::unknown("not captured"),
        };
        let catalog = Catalog::with_parts(
            vec![service],
            vec![empty_observed("helper")],
            vec![],
            vec![runtime],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownRuntimeState { .. })));
    }
}
