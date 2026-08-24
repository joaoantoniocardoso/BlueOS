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
        if let AssertedSet::Established { items } = &service.definition.edges {
            for rationaled in items.iter() {
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

    const TEST_PRESENCE: crate::version::Availability = crate::version::Availability {
        intro_commit: "0000000000000000000000000000000000000001",
        present_in_tags: &["1.0.0"],
        present_on_master: true,
        present_on_1_4_dev: true,
    };

    use crate::catalog::Catalog;
    use crate::criticality::CriticalityTier;
    use crate::id::ServiceId;
    use crate::lifecycle::Lifecycle;
    use crate::observed::ObservedFacts;
    use crate::provenance::{Asserted, AssertedSet, Evidenced, GroundedSet, Observed, ObservedSet};
    use crate::runtime::RuntimeFacts;
    use crate::service::{Service, ServiceJudgment};

    fn sample_service() -> ServiceJudgment {
        ServiceJudgment {
            id: ServiceId::Ping,
            singleton: Asserted::established(true, "test"),
            bounded_context: Asserted::established("platform", "test"),
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
            listen: ObservedSet::known(
                const {
                    &[Evidenced::new(
                        crate::id::PortRef::Literal(8000),
                        crate::provenance::Evidence {
                            file: "test.rs",
                            line: 1,
                            anchor: "",
                        },
                    )]
                },
            ),
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
        let runtime = RuntimeFacts {
            service: ServiceId::Ping,
            state_contracts: GroundedSet::unknown("not captured"),
            slo_baselines: GroundedSet::unknown("not captured"),
            resource_usage: GroundedSet::unknown("not captured"),
            platform_matrix: GroundedSet::unknown("not captured"),
            settings_mutations: GroundedSet::unknown("not captured"),
        };
        let catalog = Catalog::with_parts(
            vec![Service {
                id: ServiceId::Ping,
                observed: sample_observed(),
                definition: sample_service(),
                runtime,
            }],
            vec![],
            vec![],
        );
        let json = export_json(&catalog).expect("serialize catalog");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert!(value.is_object());
    }

    #[test]
    fn journey_types_round_trip() {
        use crate::id::{JourneyId, ServiceId};
        use crate::journey::{
            Actor, BodyKind, HttpMethod, JourneyStep, RouteRef, StepOutcome, UseCase, Visibility,
            BLAST_RADIUS_UNKNOWN,
        };
        use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

        let journey = UseCase {
            id: JourneyId::Deploy,
            summary: Grounded::known("deploy vehicle", Provenance::doc("docs/deploy.md", 1, "")),
            visibility: Grounded::known(
                Visibility::Default,
                Provenance::doc("docs/deploy.md", 2, ""),
            ),
            services: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        ServiceId::Helper,
                        Provenance::runtime("baseline-v1", "lab"),
                    )]
                },
            ),
            capability_refs: GroundedSet::unknown("not grounded"),
            preconditions: GroundedSet::unknown("not grounded"),
            steps: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        JourneyStep {
                            actor: Actor::Operator,
                            description: "call deploy",
                            route: Some(Grounded::known(
                                RouteRef {
                                    service: ServiceId::Helper,
                                    method: HttpMethod::Post,
                                    path: "/deploy",
                                    version: None,
                                },
                                Provenance::source(
                                    "core/services/ardupilot_manager/api/v1/routers/index.py",
                                    42,
                                    "",
                                ),
                            )),
                            outcome: Some(Grounded::known(
                                StepOutcome {
                                    expected_status: Some(404),
                                    body_predicate: Some("no default firmware available"),
                                    body_kind: BodyKind::Unknown,
                                    transition: None,
                                },
                                Provenance::runtime(
                                    "tests/baselines/ardupilot_manager_python_pi.json",
                                    "pi4-sitl",
                                ),
                            )),
                        },
                        Provenance::source("helper/main.py", 10, ""),
                    )]
                },
            ),
            availability: TEST_PRESENCE,
            blast_radius: BLAST_RADIUS_UNKNOWN,
            chains_from: None,
        };

        let json = serde_json::to_string(&journey).expect("serialize journey");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert!(value.is_object());
    }

    #[test]
    fn runtime_types_round_trip() {
        use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_SITL;
        use crate::id::ServiceId;
        use crate::journey::{HttpMethod, RouteRef};
        use crate::provenance::{GroundedItem, GroundedSet, Provenance};
        use crate::runtime::{
            Distribution, ResourceUsage, RuntimeFacts, SloBaseline, StateContract,
        };

        let facts = RuntimeFacts {
            service: ServiceId::ArdupilotManager,
            state_contracts: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        StateContract {
                            machine: "autopilot_lifecycle",
                            state: "running",
                            route: RouteRef {
                                service: ServiceId::ArdupilotManager,
                                method: HttpMethod::Get,
                                path: "/firmware_info",
                                version: None,
                            },
                            status: 200,
                            body_predicate: None,
                        },
                        Provenance::runtime(
                            "runtime-captures/sample.json#k",
                            RUNTIME_CAPTURE_ENV_PI4_SITL,
                        ),
                    )]
                },
            ),
            slo_baselines: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        SloBaseline {
                            route: RouteRef {
                                service: ServiceId::ArdupilotManager,
                                method: HttpMethod::Post,
                                path: "/start",
                                version: None,
                            },
                            latency_p50_ms: 12.5,
                            latency_p95_ms: 45.0,
                            latency_p99_ms: 80.0,
                            sample_size: 100,
                        },
                        Provenance::runtime(
                            "runtime-captures/sample.json#k",
                            RUNTIME_CAPTURE_ENV_PI4_SITL,
                        ),
                    )]
                },
            ),
            resource_usage: GroundedSet::known(
                const {
                    &[GroundedItem::new(
                        ResourceUsage {
                            condition: "running_navigator",
                            cpu_pct: Distribution {
                                mean: 3.2,
                                median: 2.8,
                                p95: 5.1,
                                min: 1.0,
                                max: 6.4,
                                sd: 1.2,
                            },
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
                            RUNTIME_CAPTURE_ENV_PI4_SITL,
                        ),
                    )]
                },
            ),
            platform_matrix: GroundedSet::unknown("not captured"),
            settings_mutations: GroundedSet::unknown("not captured"),
        };

        let json = serde_json::to_string(&facts).expect("serialize runtime facts");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert!(value.is_object());
    }

    #[test]
    fn schema_export_is_valid_json() {
        let schema = export_schema().expect("export schema");
        let parsed: serde_json::Value =
            serde_json::from_str(&schema).expect("schema is valid json");
        assert!(parsed.is_object());
    }
}
