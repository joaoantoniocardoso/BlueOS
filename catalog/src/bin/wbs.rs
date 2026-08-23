use std::collections::HashMap;

use blueos_catalog::domain::{ALL_AGGREGATES, DOMAINS};
use blueos_catalog::function::FunctionCatalog;
use blueos_catalog::{Aggregate, Catalog, Domain, FeatureCatalog, FeatureId};
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
    let features = FeatureCatalog::from_catalog(&catalog);
    let functions = FunctionCatalog::from_catalog(&catalog);
    let report = build_report(&features, &functions);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("serialize wbs")
        );
        return;
    }

    print_text(&report);
}

fn build_report(features: &FeatureCatalog, functions: &FunctionCatalog) -> WbsReport {
    let mut by_aggregate: HashMap<Aggregate, Vec<FeatureId>> = HashMap::new();
    for feature in features.features() {
        by_aggregate
            .entry(feature.aggregate)
            .or_default()
            .push(feature.id);
    }
    for ids in by_aggregate.values_mut() {
        ids.sort();
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
                    let mut feature_ids = by_aggregate.get(aggregate).cloned().unwrap_or_default();
                    feature_ids.sort();
                    AggregateNode {
                        id: *aggregate,
                        capabilities: feature_ids
                            .iter()
                            .map(|feature| CapabilityNode {
                                id: feature.0.as_str().to_string(),
                                functions: functions_by_capability
                                    .get(feature.0.as_str())
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
        features: features.features().len(),
        functions: functions.functions().len(),
        tree,
    }
}

fn print_text(report: &WbsReport) {
    println!("BlueOS Catalog WBS (subject axis)");
    println!("=================================");
    println!();
    println!(
        "Summary: {} domains, {} aggregates, {} features, {} functions",
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
