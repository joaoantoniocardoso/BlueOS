use blueos_catalog::{Catalog, FeatureCatalog};

fn main() {
    let catalog = Catalog::bootstrap();
    let features = FeatureCatalog::from_catalog(&catalog);

    let feature_split = features.split_consensus(&catalog);
    println!(
        "############ FEATURE GROUPINGS ({} features) ############",
        features.features().len()
    );
    for lens in &feature_split.lenses {
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
            let names: Vec<&str> = group.iter().map(|id| id.0.as_str()).collect();
            println!("     {}", names.join(","));
        }
    }
    println!(
        "\n== FEATURE CROSS-LENS CONSENSUS (lenses={}, majority>={}) ==",
        feature_split.lenses.len(),
        feature_split.majority_threshold
    );
    println!("  -- top pair agreement --");
    for pair in feature_split.pair_agreement.iter().take(25) {
        println!(
            "     {}/{}  {} <-> {}",
            pair.agree,
            pair.total,
            pair.a.0.as_str(),
            pair.b.0.as_str()
        );
    }
    println!("  -- consensus clusters (>= majority in >=2 members) --");
    for cluster in &feature_split.consensus_clusters {
        if cluster.len() < 2 {
            continue;
        }
        let names: Vec<&str> = cluster.iter().map(|id| id.0.as_str()).collect();
        println!("     {}", names.join(","));
    }
    let feature_singletons = feature_split
        .consensus_clusters
        .iter()
        .filter(|c| c.len() == 1)
        .count();
    println!("     ({feature_singletons} singletons at majority threshold)");

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
