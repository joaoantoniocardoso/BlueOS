use schemars::schema_for;

use crate::catalog::Catalog;

pub fn export_json(catalog: &Catalog) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(catalog)
}

pub fn export_schema() -> Result<String, serde_json::Error> {
    let schema = schema_for!(Catalog);
    serde_json::to_string_pretty(&schema)
}

pub fn export_mermaid(catalog: &Catalog) -> String {
    let mut output = String::from("graph LR\n");
    for service in catalog.services() {
        output.push_str(&format!("  {}[{}]\n", service.id.0, service.id.0));
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
    use crate::provenance::{Asserted, Observed};
    use crate::service::ServiceDefinition;

    fn sample_service() -> ServiceDefinition {
        ServiceDefinition {
            id: ServiceId("sample".to_string()),
            singleton: Asserted::established(true, "test"),
            bounded_context: Asserted::established("platform".to_string(), "test"),
            user_journeys: Asserted::established(vec!["deploy".to_string()], "test"),
            tier: Asserted::established(CriticalityTier::Auxiliary, "test"),
            offline_required: Asserted::established(false, "test"),
            privilege_level: Asserted::unknown("not established"),
            dangerous_operations: Asserted::unknown("not established"),
            user_confirmation: Asserted::unknown("not established"),
            capabilities: Asserted::unknown("not established"),
            authorities: Asserted::unknown("not established"),
            states: Asserted::unknown("not established"),
            edges: Asserted::unknown("not established"),
            resources: Asserted::unknown("not established"),
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
            failure_modes: Asserted::unknown("not established"),
            blast_radius: Asserted::unknown("not established"),
            compatibility_policy: Asserted::unknown("not established"),
            team: Asserted::unknown("not established"),
            adr_refs: Asserted::unknown("not established"),
        }
    }

    fn sample_observed() -> ObservedFacts {
        ObservedFacts {
            id: ServiceId("sample".to_string()),
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
    fn catalog_json_round_trips() {
        let catalog = Catalog::with_parts(vec![sample_service()], vec![sample_observed()]);
        let json = export_json(&catalog).expect("serialize catalog");
        let restored: Catalog = serde_json::from_str(&json).expect("deserialize catalog");
        assert_eq!(catalog, restored);
    }

    #[test]
    fn schema_export_is_valid_json() {
        let schema = export_schema().expect("export schema");
        let parsed: serde_json::Value =
            serde_json::from_str(&schema).expect("schema is valid json");
        assert!(parsed.is_object());
    }
}
