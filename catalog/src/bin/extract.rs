use std::path::Path;
use std::process;

use blueos_catalog::{
    check_against_observed, check_nginx_against_observed, extract_from_repo,
    extract_nginx_from_repo, Catalog,
};

fn main() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let json_mode = std::env::args().any(|arg| arg == "--json");

    let extracted = match extract_from_repo(repo_root) {
        Ok(services) => services,
        Err(err) => {
            eprintln!("extract: {err:?}");
            process::exit(2);
        }
    };

    let nginx_extracted = match extract_nginx_from_repo(repo_root) {
        Ok(locations) => locations,
        Err(err) => {
            eprintln!("extract-nginx: {err:?}");
            process::exit(2);
        }
    };

    if json_mode {
        match serde_json::to_string_pretty(&extracted) {
            Ok(json) => println!("{json}"),
            Err(err) => {
                eprintln!("extract: serialize: {err}");
                process::exit(2);
            }
        }
        return;
    }

    let catalog = Catalog::bootstrap();
    let services = catalog.services();
    let report = check_against_observed(&extracted, services);
    let nginx_report = check_nginx_against_observed(&nginx_extracted, services);
    if report.has_drift() || nginx_report.has_drift() {
        for finding in report.findings.iter().chain(nginx_report.findings.iter()) {
            eprintln!("extract-drift: {} — {}", finding.field, finding.message);
        }
        process::exit(1);
    }
}
