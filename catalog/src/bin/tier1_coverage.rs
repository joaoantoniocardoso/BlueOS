use blueos_catalog::catalog::Catalog;
use blueos_catalog::cli::percent;
use catalog_harness::runner::{tier1_get_coverage, Tier1GetCoverage};

fn main() {
    let catalog = Catalog::bootstrap();
    let coverage = tier1_get_coverage(&catalog);
    print_report(&coverage);
}

fn print_report(coverage: &Tier1GetCoverage) {
    let asserted_pct = percent(coverage.get_asserted, coverage.get_concrete);
    let concrete_pct = percent(
        coverage.get_concrete,
        coverage.get_concrete + coverage.get_templated,
    );

    println!("BlueOS catalog — Tier-1 GET coverage");
    println!();
    println!("Http-automatable journeys: {}", coverage.http_journeys);
    println!();
    println!("GET RouteRef steps:");
    println!("  Concrete (no `{{` in path): {}", coverage.get_concrete);
    println!(
        "  Asserted (expected_status set): {} ({asserted_pct:.1}%)",
        coverage.get_asserted
    );
    println!(
        "  Unasserted (missing expected_status): {}",
        coverage.get_unasserted
    );
    println!(
        "  Templated (out of Tier-1 gate): {}",
        coverage.get_templated
    );
    println!("  Concrete share of GET steps: {concrete_pct:.1}%",);
    println!();
    if coverage.unasserted.is_empty() {
        println!("Unasserted concrete GET steps: 0 (Tier-1 gate satisfied)");
    } else {
        println!(
            "Unasserted concrete GET steps ({}):",
            coverage.unasserted.len()
        );
        for (journey_id, step_index, path) in &coverage.unasserted {
            println!("  {journey_id} step {step_index} GET {path}");
        }
    }
}
