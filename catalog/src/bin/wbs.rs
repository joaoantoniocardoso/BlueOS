use std::collections::HashMap;

use blueos_catalog::domain::{ALL_AGGREGATES, DOMAINS};
use blueos_catalog::function::ActionCatalog;
use blueos_catalog::id::CapabilityId;
use blueos_catalog::{declared_capabilities, Aggregate, Catalog, Domain};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct WbsReport {
    domains: usize,
    aggregates: usize,
    features: usize,
    functions: usize,
    tree: Vec<DomainNode>,
}

#[derive(Debug, Serialize)]
struct DomainNode {
    id: Domain,
    rationale: &'static str,
    aggregates: Vec<AggregateNode>,
}

#[derive(Debug, Serialize)]
struct AggregateNode {
    id: Aggregate,
    capabilities: Vec<CapabilityNode>,
}

#[derive(Debug, Serialize)]
struct CapabilityNode {
    id: String,
    functions: Vec<String>,
}

fn main() {
    let json = std::env::args().any(|arg| arg == "--json");
    let catalog = Catalog::bootstrap();
    let capabilities = declared_capabilities(&catalog);
    let functions = ActionCatalog::from_catalog(&catalog);
    let report = build_report(&capabilities, &functions);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("serialize wbs")
        );
        return;
    }

    print_text(&report);
}

fn build_report(
    capabilities: &[blueos_catalog::DeclaredCapability],
    functions: &ActionCatalog,
) -> WbsReport {
    let mut by_aggregate: HashMap<Aggregate, Vec<CapabilityId>> = HashMap::new();
    for capability in capabilities {
        by_aggregate
            .entry(capability.aggregate)
            .or_default()
            .push(capability.id);
    }
    for ids in by_aggregate.values_mut() {
        ids.sort_by_key(|id| id.as_str());
    }

    let mut functions_by_capability: HashMap<String, Vec<String>> = HashMap::new();
    for function in functions.functions() {
        functions_by_capability
            .entry(function.capability.as_str().to_string())
            .or_default()
            .push(function.id.as_str().to_string());
    }
    for ids in functions_by_capability.values_mut() {
        ids.sort();
    }

    let tree = DOMAINS
        .iter()
        .map(|domain| DomainNode {
            id: domain.id,
            rationale: domain.rationale,
            aggregates: domain
                .aggregates
                .iter()
                .map(|aggregate| {
                    let mut capability_ids =
                        by_aggregate.get(aggregate).cloned().unwrap_or_default();
                    capability_ids.sort_by_key(|id| id.as_str());
                    AggregateNode {
                        id: *aggregate,
                        capabilities: capability_ids
                            .iter()
                            .map(|capability| CapabilityNode {
                                id: capability.as_str().to_string(),
                                functions: functions_by_capability
                                    .get(capability.as_str())
                                    .cloned()
                                    .unwrap_or_default(),
                            })
                            .collect(),
                    }
                })
                .collect(),
        })
        .collect();

    WbsReport {
        domains: DOMAINS.len(),
        aggregates: ALL_AGGREGATES.len(),
        features: capabilities.len(),
        functions: functions.functions().len(),
        tree,
    }
}

fn print_text(report: &WbsReport) {
    println!("BlueOS Catalog WBS (subject axis)");
    println!("=================================");
    println!();
    println!(
        "Summary: {} domains, {} aggregates, {} capabilities, {} functions",
        report.domains, report.aggregates, report.features, report.functions
    );
    println!();

    for domain in &report.tree {
        println!("## {}", domain.id);
        println!("rationale: {}", domain.rationale);
        for aggregate in &domain.aggregates {
            let capability_count = aggregate.capabilities.len();
            let label = if capability_count == 1 {
                "capability"
            } else {
                "capabilities"
            };
            println!("  {} ({capability_count} {label})", aggregate.id);
            for capability in &aggregate.capabilities {
                let function_count = capability.functions.len();
                let function_label = if function_count == 1 {
                    "function"
                } else {
                    "functions"
                };
                println!("    {} ({function_count} {function_label})", capability.id);
                for function in &capability.functions {
                    println!("      - {function}");
                }
            }
        }
        println!();
    }
}
