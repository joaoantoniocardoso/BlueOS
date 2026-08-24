use blueos_catalog::catalog::Catalog;
use blueos_catalog::cli::percent;
use catalog_analysis::coverage::{
    coverage_report, CoverageKind, CoverageReport, KindCounts, TRACKER_CSV_PATH,
};
use catalog_analysis::coverage_mappings::TRACKER_MAPPINGS;

fn main() {
    let catalog = Catalog::bootstrap();
    let report = coverage_report(&catalog);
    print_report(&report);
}

fn print_report(report: &CoverageReport) {
    let journey_pct = percent(report.by_kind.journey, report.total_tasks);
    let automatable_total = report.by_automatable.http
        + report.by_automatable.frontend
        + report.by_automatable.hardware
        + report.by_automatable.external_gcs
        + report.by_automatable.manual;

    println!("BlueOS Release Testing Tracker ↔ catalog coverage");
    println!("Tracker CSV: catalog/{TRACKER_CSV_PATH}");
    println!();
    println!("Total mapped tracker tasks: {}", report.total_tasks);
    println!();
    println!("By CoverageKind:");
    print_kind_counts(&report.by_kind);
    println!(
        "  Journey-mapped: {} ({journey_pct:.1}%)",
        report.by_kind.journey
    );
    println!();
    println!(
        "Journey-mapped automatable breakdown ({} journey refs):",
        automatable_total
    );
    println!("  Http:         {}", report.by_automatable.http);
    println!("  Frontend:     {}", report.by_automatable.frontend);
    println!("  Hardware:     {}", report.by_automatable.hardware);
    println!("  External GCS: {}", report.by_automatable.external_gcs);
    println!("  Manual:       {}", report.by_automatable.manual);
    println!();
    println!("Unmodeled (BlueOS gaps): {}", report.unmodeled.len());
    for task in &report.unmodeled {
        println!("  [{}] {}", task.category, task.task);
    }
    println!();
    println!("External (out of catalog scope): {}", report.external.len());
    for task in &report.external {
        println!("  [{}] {}", task.category, task.task);
    }
    println!();
    if report.unmapped.is_empty() {
        println!("Unmapped: 0 (TRACKER_MAPPINGS covers every CSV task)");
    } else {
        println!("Unmapped (need human): {}", report.unmapped.len());
        for task in &report.unmapped {
            println!("  [{}] {}", task.category, task.task);
        }
    }
    println!();
    println!("Top requirement types across mapped journeys:");
    for (label, count) in report.top_requirements.iter().take(15) {
        println!("  {count:>3}  {label}");
    }
    println!();
    println!(
        "Ignore rows: {}",
        TRACKER_MAPPINGS
            .iter()
            .filter(|entry| entry.kind == CoverageKind::Ignore)
            .count()
    );
}

fn print_kind_counts(counts: &KindCounts) {
    println!("  Journey:   {}", counts.journey);
    println!("  Unmodeled: {}", counts.unmodeled);
    println!("  External:  {}", counts.external);
    println!("  Ignore:    {}", counts.ignore);
}
