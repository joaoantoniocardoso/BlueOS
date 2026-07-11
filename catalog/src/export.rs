use schemars::schema_for;

use crate::catalog::Catalog;
use crate::cluster::{bus_label, ClusterPolicy};
use crate::provenance::AssertedSet;

pub fn export_json(catalog: &Catalog) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(catalog)
}

pub fn export_schema() -> Result<String, serde_json::Error> {
    let schema = schema_for!(Catalog);
    serde_json::to_string_pretty(&schema)
}

#[derive(serde::Serialize)]
struct ProposalsExport {
    proposals: Vec<crate::cluster::ClusterResult>,
    coupling_matrix: crate::catalog::CouplingMatrix,
    stability: crate::cluster::StabilityReport,
}

pub fn export_proposals_json(catalog: &Catalog) -> Result<String, serde_json::Error> {
    let export = ProposalsExport {
        proposals: catalog.boundary_proposals(),
        coupling_matrix: catalog.coupling_matrix(ClusterPolicy::CouplingOnly),
        stability: catalog.cluster_stability(ClusterPolicy::CouplingOnly, 10, 0.1),
    };
    serde_json::to_string_pretty(&export)
}

pub fn export_mermaid(catalog: &Catalog) -> String {
    let cluster = catalog.cluster(ClusterPolicy::CouplingOnly);
    let mut output = String::from("graph LR\n");

    for (community_idx, community) in cluster.communities.iter().enumerate() {
        output.push_str(&format!("  subgraph community_{community_idx}\n"));
        for service_id in community {
            output.push_str(&format!(
                "    {}[{}]\n",
                service_id.as_str(),
                service_id.as_str()
            ));
        }
        output.push_str("  end\n");
    }

    for service in catalog.services() {
        if let AssertedSet::Established { items } = &service.edges {
            for rationaled in items {
                let edge = &rationaled.value;
                output.push_str(&format!(
                    "  {} -->|{}| {}\n",
                    edge.from.as_str(),
                    bus_label(edge.via),
                    edge.to.as_str()
                ));
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::criticality::CriticalityTier;
    use crate::id::ServiceId;
    use crate::lifecycle::Lifecycle;
    use crate::observed::ObservedFacts;
    use crate::provenance::{Asserted, AssertedSet, Evidenced, Observed, ObservedSet};
    use crate::service::ServiceDefinition;

    fn sample_service() -> ServiceDefinition {
        ServiceDefinition {
            id: ServiceId::Ping,
            singleton: Asserted::established(true, "test"),
            bounded_context: Asserted::established("platform".to_string(), "test"),
            journey_refs: AssertedSet::unknown("not established"),
            tier: Asserted::established(CriticalityTier::Auxiliary, "test"),
            offline_required: Asserted::established(false, "test"),
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
            is_platform: Asserted::established(false, "test"),
            api_stable: Asserted::unknown("not established"),
            permissions_model: Asserted::unknown("not established"),
            failure_modes: AssertedSet::unknown("not established"),
            blast_radius: Asserted::unknown("not established"),
            compatibility_policy: Asserted::unknown("not established"),
            team: Asserted::unknown("not established"),
            adr_refs: AssertedSet::unknown("not established"),
        }
    }

    fn sample_observed() -> ObservedFacts {
        ObservedFacts {
            id: ServiceId::Ping,
            aliases: ObservedSet::unknown("not extracted"),
            kind: Observed::unknown("not extracted"),
            entrypoint: Observed::unknown("not extracted"),
            tmux_name: Observed::unknown("not extracted"),
            startup_tier: Observed::unknown("not extracted"),
            resource_limits: Observed::unknown("not extracted"),
            nice: Observed::unknown("not extracted"),
            run_as: Observed::unknown("not extracted"),
            nginx_prefixes: ObservedSet::unknown("not extracted"),
            listen: ObservedSet::known(vec![Evidenced::new(
                crate::id::PortRef::Literal(8000),
                crate::provenance::Evidence {
                    file: "test.rs".to_string(),
                    line: 1,
                },
            )]),
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
    fn catalog_json_round_trips() {
        let catalog = Catalog::with_parts(
            vec![sample_service()],
            vec![sample_observed()],
            vec![],
            vec![],
            vec![],
        );
        let json = export_json(&catalog).expect("serialize catalog");
        let restored: Catalog = serde_json::from_str(&json).expect("deserialize catalog");
        assert_eq!(catalog, restored);
    }

    #[test]
    fn journey_types_round_trip() {
        use crate::id::{JourneyId, ServiceId};
        use crate::journey::{
            Actor, HttpMethod, JourneyStep, RouteRef, StepOutcome, UserJourney, Visibility,
        };
        use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

        let journey = UserJourney {
            id: JourneyId("deploy".to_string()),
            summary: Grounded::known(
                "deploy vehicle".to_string(),
                Provenance::doc("docs/deploy.md", 1),
            ),
            visibility: Grounded::known(Visibility::Default, Provenance::doc("docs/deploy.md", 2)),
            services: GroundedSet::known(vec![GroundedItem::new(
                ServiceId::Helper,
                Provenance::runtime("baseline-v1", "lab"),
            )]),
            capability_refs: GroundedSet::unknown("not grounded"),
            preconditions: GroundedSet::unknown("not grounded"),
            steps: GroundedSet::known(vec![GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "call deploy".to_string(),
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Helper,
                            method: HttpMethod::Post,
                            path: "/deploy".to_string(),
                            version: None,
                        },
                        Provenance::source(
                            "core/services/ardupilot_manager/api/v1/routers/index.py",
                            42,
                        ),
                    )),
                    outcome: Some(Grounded::known(
                        StepOutcome {
                            expected_status: Some(404),
                            body_predicate: Some("no default firmware available".into()),
                            transition: None,
                        },
                        Provenance::runtime(
                            "tests/baselines/ardupilot_manager_python_pi.json",
                            "pi4-sitl",
                        ),
                    )),
                },
                Provenance::source("helper/main.py", 10),
            )]),
            chains_from: None,
        };

        let json = serde_json::to_string(&journey).expect("serialize journey");
        let restored: UserJourney = serde_json::from_str(&json).expect("deserialize journey");
        assert_eq!(journey, restored);
    }

    #[test]
    fn runtime_types_round_trip() {
        use crate::id::ServiceId;
        use crate::journey::{HttpMethod, RouteRef};
        use crate::provenance::{GroundedItem, GroundedSet, Provenance};
        use crate::runtime::{
            Distribution, ResourceUsage, RuntimeFacts, SloBaseline, StateContract,
        };

        let distribution = Distribution {
            mean: 3.2,
            median: 2.8,
            p95: 5.1,
            min: 1.0,
            max: 6.4,
            sd: 1.2,
        };

        let facts = RuntimeFacts {
            service: ServiceId::ArdupilotManager,
            state_contracts: GroundedSet::known(vec![GroundedItem::new(
                StateContract {
                    machine: "autopilot_lifecycle".to_string(),
                    state: "running".to_string(),
                    route: RouteRef {
                        service: ServiceId::ArdupilotManager,
                        method: HttpMethod::Get,
                        path: "/firmware_info".to_string(),
                        version: None,
                    },
                    status: 200,
                    body_predicate: None,
                },
                Provenance::runtime(
                    "runtime-captures/sample.json#k",
                    "BlueOS master, Pi 4, SITL",
                ),
            )]),
            slo_baselines: GroundedSet::known(vec![GroundedItem::new(
                SloBaseline {
                    route: RouteRef {
                        service: ServiceId::ArdupilotManager,
                        method: HttpMethod::Post,
                        path: "/start".to_string(),
                        version: None,
                    },
                    latency_p50_ms: 12.5,
                    latency_p95_ms: 45.0,
                    latency_p99_ms: 80.0,
                    sample_size: 100,
                },
                Provenance::runtime(
                    "runtime-captures/sample.json#k",
                    "BlueOS master, Pi 4, SITL",
                ),
            )]),
            resource_usage: GroundedSet::known(vec![GroundedItem::new(
                ResourceUsage {
                    condition: "running_navigator".to_string(),
                    cpu_pct: distribution.clone(),
                    rss_mb: Distribution {
                        mean: 114.0,
                        median: 112.0,
                        p95: 128.0,
                        min: 98.0,
                        max: 130.0,
                        sd: 8.5,
                    },
                    samples: 50,
                },
                Provenance::runtime(
                    "runtime-captures/sample.json#k",
                    "BlueOS master, Pi 4, SITL",
                ),
            )]),
            platform_matrix: GroundedSet::unknown("not captured"),
            settings_mutations: GroundedSet::unknown("not captured"),
        };

        let json = serde_json::to_string(&facts).expect("serialize runtime facts");
        let restored: RuntimeFacts =
            serde_json::from_str(&json).expect("deserialize runtime facts");
        assert_eq!(facts, restored);
    }

    #[test]
    fn schema_export_is_valid_json() {
        let schema = export_schema().expect("export schema");
        let parsed: serde_json::Value =
            serde_json::from_str(&schema).expect("schema is valid json");
        assert!(parsed.is_object());
    }
}
