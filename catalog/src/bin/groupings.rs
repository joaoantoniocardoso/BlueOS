use catalog_analysis::journey_group::CatalogJourneyGrouping;
use catalog_derive::feature::{capability_split_consensus, declared_capabilities};

use blueos_catalog::catalog::Catalog;

fn main() {
    let catalog = Catalog::bootstrap();
    let capabilities = declared_capabilities(&catalog);

    let capability_split = capability_split_consensus(&catalog);
    println!(
        "############ CAPABILITY GROUPINGS ({} capabilities) ############",
        capabilities.len()
    );
    for lens in &capability_split.lenses {
        let sizes: Vec<usize> = lens.groups.iter().map(|g| g.len()).collect();
        let q = lens
            .modularity
            .map(|value| format!("Q={value:.4}"))
            .unwrap_or_else(|| "label".to_string());
        println!(
            "-- {} ({q}, {} groups, sizes {sizes:?}) --",
            lens.lens,
            lens.groups.len()
        );
        for group in &lens.groups {
            if group.len() < 2 {
                continue;
            }
            let names: Vec<&str> = group.iter().map(|id| id.as_str()).collect();
            println!("     {}", names.join(","));
        }
    }
    println!(
        "\n== CAPABILITY CROSS-LENS CONSENSUS (lenses={}, majority>={}) ==",
        capability_split.lenses.len(),
        capability_split.majority_threshold
    );
    println!("  -- top pair agreement --");
    for pair in capability_split.pair_agreement.iter().take(25) {
        println!(
            "     {}/{}  {} <-> {}",
            pair.agree,
            pair.total,
            pair.a.as_str(),
            pair.b.as_str()
        );
    }
    println!("  -- consensus clusters (>= majority in >=2 members) --");
    for cluster in &capability_split.consensus_clusters {
        if cluster.len() < 2 {
            continue;
        }
        let names: Vec<&str> = cluster.iter().map(|id| id.as_str()).collect();
        println!("     {}", names.join(","));
    }
    let capability_singletons = capability_split
        .consensus_clusters
        .iter()
        .filter(|c| c.len() == 1)
        .count();
    println!("     ({capability_singletons} singletons at majority threshold)");

    let journey_split = catalog.journey_split_consensus();
    println!(
        "\n############ JOURNEY GROUPINGS ({} journeys) ############",
        catalog.journeys().len()
    );
    for lens in &journey_split.lenses {
        let sizes: Vec<usize> = lens.groups.iter().map(|g| g.len()).collect();
        let q = lens
            .modularity
            .map(|value| format!("Q={value:.4}"))
            .unwrap_or_else(|| "structural".to_string());
        println!(
            "-- {} ({q}, {} groups, sizes {sizes:?}) --",
            lens.lens,
            lens.groups.len()
        );
        for group in &lens.groups {
            if group.len() < 2 {
                continue;
            }
            let names: Vec<&str> = group.iter().map(|id| id.as_str()).collect();
            println!("     {}", names.join(","));
        }
    }
    println!(
        "\n== JOURNEY CROSS-LENS CONSENSUS (lenses={}, majority>={}) ==",
        journey_split.lenses.len(),
        journey_split.majority_threshold
    );
    println!("  -- top pair agreement --");
    for pair in journey_split.pair_agreement.iter().take(25) {
        println!(
            "     {}/{}  {} <-> {}",
            pair.agree,
            pair.total,
            pair.a.as_str(),
            pair.b.as_str()
        );
    }
    println!("  -- consensus clusters (>= majority, >=2 members) --");
    for cluster in &journey_split.consensus_clusters {
        if cluster.len() < 2 {
            continue;
        }
        let names: Vec<&str> = cluster.iter().map(|id| id.as_str()).collect();
        println!("     {}", names.join(","));
    }
    let journey_singletons = journey_split
        .consensus_clusters
        .iter()
        .filter(|c| c.len() == 1)
        .count();
    println!("     ({journey_singletons} singletons at majority threshold)");
}
