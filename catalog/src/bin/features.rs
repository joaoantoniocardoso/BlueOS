use catalog_derive::feature::{
    capability_aggregate_view, capability_journey_view, capability_view_divergence,
    declared_capabilities,
};

use blueos_catalog::catalog::Catalog;

fn main() {
    let catalog = Catalog::bootstrap();
    let capabilities = declared_capabilities(&catalog);
    let view = capability_aggregate_view(&catalog);

    for group in &view {
        println!("== {} ({}) ==", group.aggregate, group.capabilities.len());
        for id in &group.capabilities {
            println!("  {}", id.as_str());
        }
        println!();
    }

    println!(
        "total capabilities: {}, aggregates: {}",
        capabilities.len(),
        view.len()
    );

    let journey_view = capability_journey_view(&catalog);
    println!();
    println!("== View B (journey co-occurrence communities) ==");
    for community in &journey_view.communities {
        let name = community
            .members
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
            .join(",");
        println!("  {name}");
    }
    println!("  Q = {}", journey_view.modularity);
    let unreferenced: Vec<_> = journey_view
        .unreferenced_capabilities
        .iter()
        .map(|id| id.as_str())
        .collect();
    println!(
        "unreferenced by any journey ({}): {}",
        unreferenced.len(),
        unreferenced.join(",")
    );

    let divergence = capability_view_divergence(&catalog);
    println!();
    println!("== Divergence (A aggregate vs B journey) ==");
    for (left, right, bridge) in &divergence.joined_by_journey {
        println!("  {} <-> {} ({bridge})", left.as_str(), right.as_str());
    }
    println!(
        "split_by_journey pairs: {}",
        divergence.split_by_journey.len()
    );
}
