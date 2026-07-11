use blueos_catalog::{Catalog, FeatureCatalog};

fn main() {
    let catalog = Catalog::bootstrap();
    let features = FeatureCatalog::from_catalog(&catalog);
    let view = features.aggregate_view();

    for group in &view {
        println!("== {} ({}) ==", group.aggregate, group.features.len());
        for id in &group.features {
            println!("  {}", id.0);
        }
        println!();
    }

    println!(
        "total features: {}, aggregates: {}",
        features.features().len(),
        view.len()
    );

    let journey_view = features.journey_view(&catalog);
    println!();
    println!("== View B (journey co-occurrence communities) ==");
    for community in &journey_view.communities {
        let name = community
            .members
            .iter()
            .map(|id| id.0.as_str())
            .collect::<Vec<_>>()
            .join(",");
        println!("  {name}");
    }
    println!("  Q = {}", journey_view.modularity);
    let unreferenced: Vec<_> = journey_view
        .unreferenced_features
        .iter()
        .map(|id| id.0.as_str())
        .collect();
    println!(
        "unreferenced by any journey ({}): {}",
        unreferenced.len(),
        unreferenced.join(",")
    );

    let divergence = features.view_divergence(&catalog);
    println!();
    println!("== Divergence (A aggregate vs B journey) ==");
    for (left, right, bridge) in &divergence.joined_by_journey {
        println!("  {} <-> {} ({bridge})", left.0, right.0);
    }
    println!(
        "split_by_journey pairs: {}",
        divergence.split_by_journey.len()
    );
}
