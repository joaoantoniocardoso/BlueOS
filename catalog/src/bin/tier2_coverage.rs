use blueos_catalog::catalog::Catalog;
use blueos_catalog::cli::percent;
use catalog_harness::mutating_smoke::{tier2_mutating_coverage, Tier2MutatingCoverage};

fn main() {
    let catalog = Catalog::bootstrap();
    let coverage = tier2_mutating_coverage(&catalog);
    print_report(&coverage);
}

fn print_report(coverage: &Tier2MutatingCoverage) {
    let covered = coverage
        .eligible_count
        .saturating_sub(coverage.missing.len());
    let pct = percent(covered, coverage.eligible_count);

    println!("BlueOS catalog — Tier-2 mutating smoke coverage");
    println!();
    println!("Tier-2 eligible journeys: {}", coverage.eligible_count);
    println!(
        "Allowlisted (MUTATING_SMOKE_ENTRIES): {}",
        coverage.allowlisted_count
    );
    println!("Covered: {covered} ({pct:.1}%)");
    println!("Missing: {}", coverage.missing.len());
    println!("Hard-excluded: {}", coverage.excluded.len());
    println!();
    if coverage.missing.is_empty() {
        println!("Missing journeys: 0 (Tier-2 gate satisfied)");
    } else {
        println!("Missing journeys (eligible but not allowlisted):");
        for journey_id in &coverage.missing {
            println!("  {journey_id}");
        }
    }
    println!();
    if coverage.excluded.is_empty() {
        println!("Hard-excluded journeys: 0");
    } else {
        println!("Hard-excluded journeys (not coverage failures):");
        for journey_id in &coverage.excluded {
            println!("  {journey_id}");
        }
    }
}
