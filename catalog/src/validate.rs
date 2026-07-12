use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::catalog::Catalog;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::UserJourney;
use crate::page::{ConsumeTarget, PageId};
use crate::provenance::{AssertedSet, GroundedSet, ObservedSet};
use crate::resource::ResourceOwnership;
use crate::service::{Authority, Service, ServiceDefinition};
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
    #[error(
        "runtime facts reference unknown state {state} in machine {machine} for service {service}"
    )]
    UnknownRuntimeState {
        service: String,
        machine: String,
        state: String,
    },
    #[error("page {page} consumes unknown service {service}")]
    UnknownPageService { page: String, service: String },
    #[error("page {page} has empty frontend feature capability id")]
    EmptyPageFrontendFeature { page: String },
    #[error("duplicate page id {page}")]
    DuplicatePageId { page: String },
}

pub fn validate(catalog: &Catalog) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    errors.extend(check_exclusive_resources(catalog));
    errors.extend(check_conflicting_authorities(catalog));
    errors.extend(check_edge_targets(catalog));
    errors.extend(check_service_journey_refs(catalog));
    errors.extend(check_journey_references(catalog));
    check_runtime_references(catalog, &mut errors);
    errors.extend(check_page_references(catalog));
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
        if let ObservedSet::Known { items } = &service.observed.listen {
            for port_ref in items.iter().map(|e| &e.value) {
                if let crate::id::PortRef::Literal(port) = port_ref {
                    let key = format!("port:{port}");
                    track_exclusive_claim(&mut claims, &mut errors, &service.id, key);
                }
            }
        }

        if let AssertedSet::Established { items } = &service.definition.resources {
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
                first: existing.to_string(),
                second: service_id.to_string(),
            });
        }
    } else {
        claims.insert(key, *service_id);
    }
}

fn check_conflicting_authorities(catalog: &Catalog) -> Vec<ValidationError> {
    let mut claims: HashMap<String, ServiceId> = HashMap::new();
    let mut errors = Vec::new();

    for service in catalog.services() {
        if let AssertedSet::Established { items } = &service.definition.authorities {
            for authority in items.iter().map(|a| &a.value) {
                let key = authority_key(authority);
                if let Some(existing) = claims.get(&key) {
                    if existing != &service.id {
                        errors.push(ValidationError::ConflictingAuthority {
                            authority: key,
                            first: existing.to_string(),
                            second: service.id.to_string(),
                        });
                    }
                } else {
                    claims.insert(key, service.id);
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
        if let AssertedSet::Established { items } = &service.definition.edges {
            for edge in items.iter().map(|e| &e.value) {
                if !known_ids.contains(&edge.to) {
                    errors.push(ValidationError::UnknownEdgeTarget {
                        from: service.id.to_string(),
                        to: edge.to.to_string(),
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
        if let AssertedSet::Established { items } = &service.definition.journey_refs {
            for journey_ref in items.iter().map(|j| &j.value) {
                if !known_journey_ids.contains(journey_ref) {
                    errors.push(ValidationError::UnknownServiceJourneyRef {
                        service: service.id.to_string(),
                        journey: journey_ref.to_string(),
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
        let journey_id = journey.id.to_string();
        let participating = participating_service_ids(journey);

        if let Some(chains_from) = &journey.chains_from {
            if !known_journey_ids.contains(chains_from) {
                errors.push(ValidationError::UnknownJourneyChain {
                    journey: journey_id.clone(),
                    chains_from: chains_from.to_string(),
                });
            }
        }

        if let GroundedSet::Known { items } = &journey.services {
            for item in items.iter() {
                if !known_service_ids.contains(&item.value) {
                    errors.push(ValidationError::UnknownJourneyService {
                        journey: journey_id.clone(),
                        service: item.value.to_string(),
                    });
                }
            }
        }

        if let GroundedSet::Known { items } = &journey.capability_refs {
            for item in items.iter() {
                if !capability_in_participating_services(
                    &item.value,
                    &participating,
                    &service_index,
                ) && !is_frontend_capability(&item.value)
                {
                    errors.push(ValidationError::UnknownJourneyCapability {
                        journey: journey_id.clone(),
                        capability: item.value.to_string(),
                    });
                }
            }
        }

        if let GroundedSet::Known { items } = &journey.steps {
            for item in items.iter() {
                if let Some(crate::provenance::Grounded::Known { value: route, .. }) =
                    &item.value.route
                {
                    if !known_service_ids.contains(&route.service) {
                        errors.push(ValidationError::UnknownJourneyService {
                            journey: journey_id.clone(),
                            service: route.service.to_string(),
                        });
                    }
                }
                if let Some(crate::provenance::Grounded::Known { value: outcome, .. }) =
                    &item.value.outcome
                {
                    if let Some(transition) = &outcome.transition {
                        for state in [&transition.from, &transition.to] {
                            if !state_in_participating_services(
                                transition.machine,
                                state,
                                &participating,
                                &service_index,
                            ) {
                                errors.push(ValidationError::UnknownJourneyState {
                                    journey: journey_id.clone(),
                                    machine: transition.machine.to_string(),
                                    state: state.to_string(),
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

fn is_frontend_capability(capability: &CapabilityId) -> bool {
    crate::capability::frontend_capability_def(*capability).is_some()
}

fn capability_in_participating_services(
    capability: &CapabilityId,
    participating: &[&ServiceId],
    service_index: &HashMap<&ServiceId, &Service>,
) -> bool {
    participating.iter().any(|service_id| {
        service_index
            .get(service_id)
            .and_then(|service| match &service.definition.capabilities {
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
    service_index: &HashMap<&ServiceId, &Service>,
) -> bool {
    participating.iter().any(|service_id| {
        service_index
            .get(service_id)
            .and_then(|service| match &service.definition.states {
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
        .any(|item| item.value.name == machine && item.value.states.contains(&state))
}

fn check_runtime_references(catalog: &Catalog, errors: &mut Vec<ValidationError>) {
    let service_index = catalog.service_index();

    for service in catalog.services() {
        let service_id = service.id.to_string();
        if let GroundedSet::Known { items } = &service.runtime.state_contracts {
            for item in items.iter() {
                let contract = &item.value;
                if !state_in_service(
                    &service.id,
                    contract.machine,
                    contract.state,
                    &service_index,
                ) {
                    errors.push(ValidationError::UnknownRuntimeState {
                        service: service_id.clone(),
                        machine: contract.machine.to_string(),
                        state: contract.state.to_string(),
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
    service_index: &HashMap<&ServiceId, &Service>,
) -> bool {
    service_index
        .get(service_id)
        .and_then(|service| match &service.definition.states {
            AssertedSet::Established { items } => Some(state_in_machines(machine, state, items)),
            AssertedSet::Unknown { .. } => None,
        })
        .unwrap_or(false)
}

fn check_page_references(catalog: &Catalog) -> Vec<ValidationError> {
    if catalog.pages().is_empty() {
        return Vec::new();
    }

    let known_service_ids: HashSet<&ServiceId> = catalog.services().iter().map(|s| &s.id).collect();
    let mut seen_page_ids: HashSet<&PageId> = HashSet::new();
    let mut errors = Vec::new();

    for page in catalog.pages() {
        let page_id = page.id.to_string();

        if !seen_page_ids.insert(&page.id) {
            errors.push(ValidationError::DuplicatePageId {
                page: page_id.clone(),
            });
        }

        if let ObservedSet::Known { items } = &page.consumes {
            for item in items.iter() {
                if let ConsumeTarget::Service(id) = &item.value.service {
                    if !known_service_ids.contains(id) {
                        errors.push(ValidationError::UnknownPageService {
                            page: page_id.clone(),
                            service: id.to_string(),
                        });
                    }
                }
            }
        }

        if let AssertedSet::Established { items } = &page.frontend_features {
            for item in items.iter() {
                if item.value.as_str().is_empty() {
                    errors.push(ValidationError::EmptyPageFrontendFeature {
                        page: page_id.clone(),
                    });
                }
            }
        }
    }

    errors
}

fn check_coverage_gate(catalog: &Catalog) -> Vec<ValidationError> {
    let unknown_count: usize = catalog
        .services()
        .iter()
        .map(|service| count_unknown_in_service(&service.definition))
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
    use crate::page::{ConsumeTarget, Page, PageId, PageServiceCall};
    use crate::provenance::{
        Asserted, AssertedSet, Evidence, Evidenced, Grounded, GroundedItem, GroundedSet, Observed,
        ObservedSet, Provenance, Rationaled,
    };
    use crate::resource::{Resource, ResourceOwnership};
    use crate::runtime::{RuntimeFacts, StateContract};
    use crate::service::Service;
    use crate::state::StateMachine;

    const fn evidence() -> Evidence {
        Evidence {
            file: "test.rs",
            line: 1,
        }
    }

    fn empty_service(id: ServiceId) -> ServiceDefinition {
        ServiceDefinition {
            id,
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

    fn empty_observed(id: ServiceId) -> ObservedFacts {
        ObservedFacts {
            id,
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

    fn empty_runtime(id: ServiceId) -> RuntimeFacts {
        RuntimeFacts {
            service: id,
            state_contracts: GroundedSet::unknown("not captured"),
            slo_baselines: GroundedSet::unknown("not captured"),
            resource_usage: GroundedSet::unknown("not captured"),
            platform_matrix: GroundedSet::unknown("not captured"),
            settings_mutations: GroundedSet::unknown("not captured"),
        }
    }

    fn svc(
        id: ServiceId,
        observed: ObservedFacts,
        definition: ServiceDefinition,
        runtime: RuntimeFacts,
    ) -> Service {
        Service {
            id,
            observed,
            definition,
            runtime,
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
        let service = empty_service(ServiceId::Helper);
        let observed = empty_observed(ServiceId::Helper);
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                observed,
                service,
                empty_runtime(ServiceId::Helper),
            )],
            vec![],
            vec![],
        );
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn duplicate_exclusive_port_fails() {
        let mut service_a = empty_service(ServiceId::Ping);
        service_a.resources = AssertedSet::established(
            const {
                &[Rationaled::new(
                    Resource {
                        path: PathRef("/dev/ttyUSB0"),
                        ownership: ResourceOwnership::Exclusive,
                    },
                    "test",
                )]
            },
        );
        let mut service_b = empty_service(ServiceId::Beacon);
        service_b.resources = AssertedSet::established(
            const {
                &[Rationaled::new(
                    Resource {
                        path: PathRef("/dev/ttyUSB0"),
                        ownership: ResourceOwnership::Exclusive,
                    },
                    "test",
                )]
            },
        );

        let mut observed_a = empty_observed(ServiceId::Ping);
        observed_a.listen = ObservedSet::known(
            const {
                &[Evidenced::new(
                    crate::id::PortRef::Literal(8000),
                    evidence(),
                )]
            },
        );
        let mut observed_b = empty_observed(ServiceId::Beacon);
        observed_b.listen = ObservedSet::known(
            const {
                &[Evidenced::new(
                    crate::id::PortRef::Literal(8000),
                    evidence(),
                )]
            },
        );

        let catalog = Catalog::with_parts(
            vec![
                svc(
                    ServiceId::Ping,
                    observed_a,
                    service_a,
                    empty_runtime(ServiceId::Ping),
                ),
                svc(
                    ServiceId::Beacon,
                    observed_b,
                    service_b,
                    empty_runtime(ServiceId::Beacon),
                ),
            ],
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
        let mut service_a = empty_service(ServiceId::Ping);
        service_a.edges = AssertedSet::established(
            const {
                &[Rationaled::new(
                    Edge {
                        from: ServiceId::Ping,
                        to: ServiceId::Zenohd,
                        via: Bus::Rest,
                        sync: SyncMode::Sync,
                        endpoint: "/helper/",
                        purpose: "call helper",
                        required_at_boot: false,
                        failure_impact: FailureImpact::Degraded,
                    },
                    "test",
                )]
            },
        );

        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Ping,
                empty_observed(ServiceId::Ping),
                service_a,
                empty_runtime(ServiceId::Ping),
            )],
            vec![],
            vec![],
        );
        let result = catalog.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownEdgeTarget { .. })));
    }

    fn valid_journey_service(id: ServiceId) -> ServiceDefinition {
        let mut service = empty_service(id);
        service.capabilities =
            AssertedSet::established(const { &[Rationaled::new(CapabilityId::Deploy, "test")] });
        service.states = AssertedSet::established(
            const {
                &[Rationaled::new(
                    StateMachine {
                        name: "lifecycle",
                        states: &["idle", "running"],
                        boot_state: "idle",
                        degraded_when: &[],
                    },
                    "test",
                )]
            },
        );
        service
    }

    fn valid_journey() -> UserJourney {
        UserJourney {
            id: JourneyId::Deploy,
            summary: Grounded::known("deploy vehicle", Provenance::doc("docs/deploy.md", 1)),
            visibility: Grounded::known(Visibility::Default, Provenance::doc("docs/deploy.md", 2)),
            services: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        ServiceId::Helper,
                        Provenance::doc("docs/deploy.md", 3),
                    )]
                },
            ),
            capability_refs: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        CapabilityId::Deploy,
                        Provenance::doc("docs/deploy.md", 4),
                    )]
                },
            ),
            preconditions: GroundedSet::unknown("not grounded"),
            steps: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        JourneyStep {
                            actor: Actor::Operator,
                            description: "call deploy endpoint",
                            route: Some(Grounded::known(
                                RouteRef {
                                    service: ServiceId::Helper,
                                    method: HttpMethod::Post,
                                    path: "/deploy",
                                    version: Some("v1"),
                                },
                                Provenance::doc("docs/deploy.md", 5),
                            )),
                            outcome: Some(Grounded::known(
                                StepOutcome {
                                    expected_status: Some(200),
                                    body_predicate: None,
                                    transition: Some(StateTransition {
                                        machine: "lifecycle",
                                        from: "idle",
                                        to: "running",
                                    }),
                                },
                                Provenance::runtime("tests/baselines/helper.json", "lab"),
                            )),
                        },
                        Provenance::source("helper/main.py", 10),
                    )]
                },
            ),
            chains_from: None,
        }
    }

    #[test]
    fn valid_journey_passes_validate() {
        let service = valid_journey_service(ServiceId::Helper);
        let observed = empty_observed(ServiceId::Helper);
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                observed,
                service,
                empty_runtime(ServiceId::Helper),
            )],
            vec![valid_journey()],
            vec![],
        );
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn unknown_journey_service_ref_fails() {
        let service = valid_journey_service(ServiceId::Helper);
        let journey = UserJourney {
            services: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        ServiceId::Zenohd,
                        Provenance::doc("docs/deploy.md", 1),
                    )]
                },
            ),
            ..valid_journey()
        };
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                empty_observed(ServiceId::Helper),
                service,
                empty_runtime(ServiceId::Helper),
            )],
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
        let service = valid_journey_service(ServiceId::Helper);
        let journey = UserJourney {
            steps: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        JourneyStep {
                            actor: Actor::Operator,
                            description: "call deploy endpoint",
                            route: Some(Grounded::known(
                                RouteRef {
                                    service: ServiceId::Zenohd,
                                    method: HttpMethod::Post,
                                    path: "/deploy",
                                    version: Some("v1"),
                                },
                                Provenance::doc("docs/deploy.md", 5),
                            )),
                            outcome: Some(Grounded::known(
                                StepOutcome {
                                    expected_status: Some(200),
                                    body_predicate: None,
                                    transition: Some(StateTransition {
                                        machine: "lifecycle",
                                        from: "idle",
                                        to: "running",
                                    }),
                                },
                                Provenance::runtime("tests/baselines/helper.json", "lab"),
                            )),
                        },
                        Provenance::source("helper/main.py", 10),
                    )]
                },
            ),
            ..valid_journey()
        };
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                empty_observed(ServiceId::Helper),
                service,
                empty_runtime(ServiceId::Helper),
            )],
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
        let service = valid_journey_service(ServiceId::Helper);
        let mut journey = valid_journey();
        journey.capability_refs = GroundedSet::known(
            const {
                &[GroundedItem::new(
                    CapabilityId::FlashFirmware,
                    Provenance::doc("docs/deploy.md", 1),
                )]
            },
        );
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                empty_observed(ServiceId::Helper),
                service,
                empty_runtime(ServiceId::Helper),
            )],
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
        let service = valid_journey_service(ServiceId::Helper);
        let journey = UserJourney {
            steps: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        JourneyStep {
                            actor: Actor::Operator,
                            description: "call deploy endpoint",
                            route: Some(Grounded::known(
                                RouteRef {
                                    service: ServiceId::Helper,
                                    method: HttpMethod::Post,
                                    path: "/deploy",
                                    version: Some("v1"),
                                },
                                Provenance::doc("docs/deploy.md", 5),
                            )),
                            outcome: Some(Grounded::known(
                                StepOutcome {
                                    expected_status: Some(200),
                                    body_predicate: None,
                                    transition: Some(StateTransition {
                                        machine: "lifecycle",
                                        from: "idle",
                                        to: "missing",
                                    }),
                                },
                                Provenance::runtime("tests/baselines/helper.json", "lab"),
                            )),
                        },
                        Provenance::source("helper/main.py", 10),
                    )]
                },
            ),
            ..valid_journey()
        };
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                empty_observed(ServiceId::Helper),
                service,
                empty_runtime(ServiceId::Helper),
            )],
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
        let service = valid_journey_service(ServiceId::Helper);
        let journey = UserJourney {
            steps: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        JourneyStep {
                            actor: Actor::Operator,
                            description: "call deploy endpoint",
                            route: Some(Grounded::known(
                                RouteRef {
                                    service: ServiceId::Helper,
                                    method: HttpMethod::Post,
                                    path: "/deploy",
                                    version: Some("v1"),
                                },
                                Provenance::doc("docs/deploy.md", 5),
                            )),
                            outcome: Some(Grounded::known(
                                StepOutcome {
                                    expected_status: Some(200),
                                    body_predicate: None,
                                    transition: Some(StateTransition {
                                        machine: "nonexistent",
                                        from: "idle",
                                        to: "running",
                                    }),
                                },
                                Provenance::runtime("tests/baselines/helper.json", "lab"),
                            )),
                        },
                        Provenance::source("helper/main.py", 10),
                    )]
                },
            ),
            ..valid_journey()
        };
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                empty_observed(ServiceId::Helper),
                service,
                empty_runtime(ServiceId::Helper),
            )],
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
        let service = valid_journey_service(ServiceId::Helper);
        let mut journey = valid_journey();
        journey.chains_from = Some(JourneyId::RebootOnboardComputer);
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                empty_observed(ServiceId::Helper),
                service,
                empty_runtime(ServiceId::Helper),
            )],
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
        let mut service = empty_service(ServiceId::Helper);
        service.journey_refs = AssertedSet::established(
            const { &[Rationaled::new(JourneyId::RebootOnboardComputer, "test")] },
        );
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                empty_observed(ServiceId::Helper),
                service,
                empty_runtime(ServiceId::Helper),
            )],
            vec![],
            vec![],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownServiceJourneyRef { .. })));
    }

    #[test]
    fn unknown_runtime_state_fails() {
        let service = valid_journey_service(ServiceId::Helper);
        let runtime = RuntimeFacts {
            service: ServiceId::Helper,
            state_contracts: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        StateContract {
                            machine: "lifecycle",
                            state: "missing",
                            route: RouteRef {
                                service: ServiceId::Helper,
                                method: HttpMethod::Get,
                                path: "/status",
                                version: None,
                            },
                            status: 200,
                            body_predicate: None,
                        },
                        Provenance::runtime("runtime-captures/sample.json#k", "lab"),
                    )]
                },
            ),
            slo_baselines: GroundedSet::unknown("not captured"),
            resource_usage: GroundedSet::unknown("not captured"),
            platform_matrix: GroundedSet::unknown("not captured"),
            settings_mutations: GroundedSet::unknown("not captured"),
        };
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                empty_observed(ServiceId::Helper),
                service,
                runtime,
            )],
            vec![],
            vec![],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownRuntimeState { .. })));
    }

    const fn consume(target: ConsumeTarget) -> Evidenced<PageServiceCall> {
        Evidenced::new(
            PageServiceCall {
                service: target,
                endpoint: "GET /status",
                purpose: "load page data",
            },
            evidence(),
        )
    }

    fn sample_page(consumes: ObservedSet<PageServiceCall>) -> Page {
        Page {
            id: PageId::VehicleSetup,
            route: Observed::known("/vehicle/setup", evidence()),
            name: Observed::known("Vehicle Setup", evidence()),
            component: Observed::known("core/frontend/src/views/VehicleSetupView.vue", evidence()),
            menu_title: Observed::unknown("not in menu"),
            advanced_only: Observed::unknown("not in menu"),
            stores: ObservedSet::unknown("not extracted"),
            consumes,
            frontend_features: AssertedSet::established(
                const {
                    &[Rationaled::new(
                        CapabilityId::CalibrateAccelerometer,
                        "client-side only",
                    )]
                },
            ),
            client_state: AssertedSet::unknown("not established"),
        }
    }

    #[test]
    fn valid_page_passes_validate() {
        let service = empty_service(ServiceId::Helper);
        let observed = empty_observed(ServiceId::Helper);
        let page = sample_page(ObservedSet::known(
            const { &[consume(ConsumeTarget::Service(ServiceId::Helper))] },
        ));
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                observed,
                service,
                empty_runtime(ServiceId::Helper),
            )],
            vec![],
            vec![page],
        );
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn unknown_page_service_call_fails() {
        let service = empty_service(ServiceId::Helper);
        let observed = empty_observed(ServiceId::Helper);
        let page = sample_page(ObservedSet::known(
            const { &[consume(ConsumeTarget::Service(ServiceId::Zenohd))] },
        ));
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                observed,
                service,
                empty_runtime(ServiceId::Helper),
            )],
            vec![],
            vec![page],
        );
        let errors = catalog.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownPageService { .. })));
    }

    #[test]
    fn external_page_service_call_passes() {
        let service = empty_service(ServiceId::Helper);
        let observed = empty_observed(ServiceId::Helper);
        let page = sample_page(ObservedSet::known(
            const { &[consume(ConsumeTarget::External)] },
        ));
        let catalog = Catalog::with_parts(
            vec![svc(
                ServiceId::Helper,
                observed,
                service,
                empty_runtime(ServiceId::Helper),
            )],
            vec![],
            vec![page],
        );
        assert!(catalog.validate().is_ok());
    }
}
