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
                .map(|id| id.0.as_str())
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
            println!("  {} <-> {} co_occurrence={rate}", left.0, right.0);
        }
    }
    println!();

    println!("== mermaid (CouplingOnly communities) ==");
    println!("{}", export_mermaid(&catalog));
}
