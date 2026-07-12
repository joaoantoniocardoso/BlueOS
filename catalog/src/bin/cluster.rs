use blueos_catalog::{export_mermaid, Catalog, ClusterPolicy};

fn main() {
    let catalog = Catalog::bootstrap();

    for policy in [
        ClusterPolicy::CouplingOnly,
        ClusterPolicy::CouplingTrust,
        ClusterPolicy::CouplingDomain,
    ] {
        let result = catalog.cluster(policy);
        println!("== {:?} ==", result.policy);
        for community in &result.communities {
            let name = community
                .iter()
                .map(|id| id.as_str())
                .collect::<Vec<_>>()
                .join(",");
            println!("  {name}");
        }
        println!("  Q = {}", result.modularity);
        println!();
    }

    let stability = catalog.cluster_stability(ClusterPolicy::CouplingOnly, 10, 0.1);
    println!(
        "== CouplingOnly stability (runs={}, jitter={}) ==",
        stability.runs, stability.jitter
    );
    if stability.unstable_pairs.is_empty() {
        println!("  (no unstable pairs)");
    } else {
        for (left, right, rate) in &stability.unstable_pairs {
            println!(
                "  {} <-> {} co_occurrence={rate}",
                left.as_str(),
                right.as_str()
            );
        }
    }
    println!();

    println!("== mermaid (CouplingOnly communities) ==");
    println!("{}", export_mermaid(&catalog));
    println!();

    let consensus = catalog.split_consensus();
    println!("== SPLIT LENSES (all approaches) ==");
    for lens in &consensus.lenses {
        let sizes: Vec<usize> = lens.communities.iter().map(|c| c.len()).collect();
        println!(
            "-- {} (Q={:.4}, {} groups, sizes {:?}) --",
            lens.lens,
            lens.modularity,
            lens.communities.len(),
            sizes
        );
        for community in &lens.communities {
            if community.len() < 2 {
                continue;
            }
            let name = community
                .iter()
                .map(|id| id.as_str())
                .collect::<Vec<_>>()
                .join(",");
            println!("     {name}");
        }
    }
    println!();

    println!(
        "== CROSS-LENS CONSENSUS (total lenses={}, majority>={}) ==",
        consensus.lenses.len(),
        consensus.majority_threshold
    );
    println!("  -- pair agreement (how many lenses group the pair) --");
    for pair in consensus.pair_agreement.iter().take(25) {
        println!(
            "     {}/{}  {} <-> {}",
            pair.agree,
            pair.total,
            pair.a.as_str(),
            pair.b.as_str()
        );
    }
    println!("  -- consensus clusters (majority-agreed pairs, connected components) --");
    for cluster in &consensus.consensus_clusters {
        if cluster.len() < 2 {
            continue;
        }
        let name = cluster
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
            .join(",");
        println!("     {name}");
    }
    let singletons = consensus
        .consensus_clusters
        .iter()
        .filter(|c| c.len() == 1)
        .count();
    println!("     ({singletons} singletons at majority threshold)");
}
