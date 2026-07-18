use std::collections::HashMap;

use blueos_catalog::domain::{ALL_AGGREGATES, DOMAINS};
use blueos_catalog::{Aggregate, Catalog, Domain, FeatureCatalog, FeatureId};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct WbsReport {
    domains: usize,
    aggregates: usize,
    features: usize,
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
    features: Vec<String>,
}

fn main() {
    let json = std::env::args().any(|arg| arg == "--json");
    let catalog = Catalog::bootstrap();
    let features = FeatureCatalog::from_catalog(&catalog);
    let report = build_report(&features);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("serialize wbs")
        );
        return;
    }

    print_text(&report);
}

fn build_report(features: &FeatureCatalog) -> WbsReport {
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
                        features: feature_ids
                            .iter()
                            .map(|id| id.0.as_str().to_string())
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
        tree,
    }
}

fn print_text(report: &WbsReport) {
    println!("BlueOS Catalog WBS (subject axis)");
    println!("=================================");
    println!();
    println!(
        "Summary: {} domains, {} aggregates, {} features",
        report.domains, report.aggregates, report.features
    );
    println!();

    for domain in &report.tree {
        println!("## {}", domain.id);
        println!("rationale: {}", domain.rationale);
        for aggregate in &domain.aggregates {
            let count = aggregate.features.len();
            let label = if count == 1 { "feature" } else { "features" };
            println!("  {} ({count} {label})", aggregate.id);
            for feature in &aggregate.features {
                println!("    - {feature}");
            }
        }
        println!();
    }
}
