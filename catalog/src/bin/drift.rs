use blueos_catalog::{diff_catalog, diff_catalog_runtime, Catalog};

fn main() {
    let catalog = Catalog::bootstrap();
    let observed_report = diff_catalog(catalog.services());
    let runtime_report = diff_catalog_runtime(catalog.services());

    let findings: Vec<_> = observed_report
        .findings
        .into_iter()
        .chain(runtime_report.findings)
        .collect();

    if findings.is_empty() {
        return;
    }

    for finding in &findings {
        eprintln!("drift: {} — {}", finding.field, finding.message);
    }
    std::process::exit(1);
}
