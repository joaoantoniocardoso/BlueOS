use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::catalog::Catalog;
use crate::id::ServiceId;
use crate::provenance::Observed;
use crate::resource::ResourceOwnership;
use crate::service::{Authority, ServiceDefinition};

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
}

pub fn validate(catalog: &Catalog) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    errors.extend(check_exclusive_resources(catalog));
    errors.extend(check_conflicting_authorities(catalog));
    errors.extend(check_edge_targets(catalog));
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
            if let Observed::Known { value: ports, .. } = &observed.listen {
                for port_ref in ports {
                    if let crate::id::PortRef::Literal(port) = port_ref {
                        let key = format!("port:{port}");
                        track_exclusive_claim(&mut claims, &mut errors, &service.id, key);
                    }
                }
            }
        }

        if let Observed::Known {
            value: resources, ..
        } = &service.resources
        {
            for resource in resources {
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
        if let Observed::Known {
            value: authorities, ..
        } = &service.authorities
        {
            for authority in authorities {
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
        if let Observed::Known { value: edges, .. } = &service.edges {
            for edge in edges {
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
    count += service.user_journeys.is_unknown() as usize;
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
    use crate::id::{PathRef, ServiceId};
    use crate::lifecycle::Lifecycle;
    use crate::observed::ObservedFacts;
    use crate::provenance::{Evidence, Observed};
    use crate::resource::{Resource, ResourceOwnership};

    fn evidence() -> Evidence {
        Evidence {
            file: "test.rs".to_string(),
            line: 1,
        }
    }

    fn empty_service(id: &str) -> ServiceDefinition {
        ServiceDefinition {
            id: ServiceId(id.to_string()),
            singleton: Observed::unknown("not established"),
            bounded_context: Observed::unknown("not established"),
            user_journeys: Observed::unknown("not established"),
            tier: Observed::unknown("not established"),
            offline_required: Observed::unknown("not established"),
            privilege_level: Observed::unknown("not established"),
            dangerous_operations: Observed::unknown("not established"),
            user_confirmation: Observed::unknown("not established"),
            capabilities: Observed::unknown("not established"),
            authorities: Observed::unknown("not established"),
            states: Observed::unknown("not established"),
            edges: Observed::unknown("not established"),
            resources: Observed::unknown("not established"),
            lifecycle: Lifecycle {
                triggers: Observed::unknown("not established"),
                ordered_after: Observed::unknown("not established"),
                ordered_before: Observed::unknown("not established"),
                shutdown: Observed::unknown("not established"),
                upgrade_behavior: Observed::unknown("not established"),
            },
            health: Observed::unknown("not established"),
            is_platform: Observed::unknown("not established"),
            api_stable: Observed::unknown("not established"),
            permissions_model: Observed::unknown("not established"),
            failure_modes: Observed::unknown("not established"),
            blast_radius: Observed::unknown("not established"),
            compatibility_policy: Observed::unknown("not established"),
            team: Observed::unknown("not established"),
            adr_refs: Observed::unknown("not established"),
        }
    }

    fn empty_observed(id: &str) -> ObservedFacts {
        ObservedFacts {
            id: ServiceId(id.to_string()),
            aliases: Observed::unknown("not extracted"),
            kind: Observed::unknown("not extracted"),
            entrypoint: Observed::unknown("not extracted"),
            tmux_name: Observed::unknown("not extracted"),
            startup_tier: Observed::unknown("not extracted"),
            resource_limits: Observed::unknown("not extracted"),
            nice: Observed::unknown("not extracted"),
            run_as: Observed::unknown("not extracted"),
            nginx_prefixes: Observed::unknown("not extracted"),
            listen: Observed::unknown("not extracted"),
            git_path: Observed::unknown("not extracted"),
            interfaces: Observed::unknown("not extracted"),
            resources: Observed::unknown("not extracted"),
            lifecycle: Observed::unknown("not extracted"),
            logs_path: Observed::unknown("not extracted"),
            zenoh_log_topic: Observed::unknown("not extracted"),
            sentry: Observed::unknown("not extracted"),
            openapi_refs: Observed::unknown("not extracted"),
        }
    }

    #[test]
    fn empty_catalog_passes_validate() {
        let catalog = Catalog::new();
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn minimal_catalog_with_unknown_fields_passes() {
        let service = empty_service("helper");
        let observed = empty_observed("helper");
        let catalog = Catalog::with_parts(vec![service], vec![observed]);
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn duplicate_exclusive_port_fails() {
        let mut service_a = empty_service("a");
        service_a.resources = Observed::known(
            vec![Resource {
                path: PathRef("/dev/ttyUSB0".to_string()),
                ownership: ResourceOwnership::Exclusive,
            }],
            evidence(),
        );
        let mut service_b = empty_service("b");
        service_b.resources = Observed::known(
            vec![Resource {
                path: PathRef("/dev/ttyUSB0".to_string()),
                ownership: ResourceOwnership::Exclusive,
            }],
            evidence(),
        );

        let mut observed_a = empty_observed("a");
        observed_a.listen = Observed::known(vec![crate::id::PortRef::Literal(8000)], evidence());
        let mut observed_b = empty_observed("b");
        observed_b.listen = Observed::known(vec![crate::id::PortRef::Literal(8000)], evidence());

        let catalog = Catalog::with_parts(vec![service_a, service_b], vec![observed_a, observed_b]);
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
        service_a.edges = Observed::known(
            vec![Edge {
                from: ServiceId("a".to_string()),
                to: ServiceId("missing".to_string()),
                via: Bus::Rest,
                sync: SyncMode::Sync,
                endpoint: "/helper/".to_string(),
                purpose: "call helper".to_string(),
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            }],
            evidence(),
        );

        let catalog = Catalog::with_parts(vec![service_a], vec![empty_observed("a")]);
        let result = catalog.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownEdgeTarget { .. })));
    }
}
