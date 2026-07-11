use blueos_catalog::{diff_catalog, Catalog};

fn main() {
    let catalog = Catalog::bootstrap();
    let report = diff_catalog(catalog.services(), catalog.observed());
    if report.has_drift() {
        for finding in &report.findings {
            eprintln!("drift: {} — {}", finding.field, finding.message);
        }
        std::process::exit(1);
    }
}
