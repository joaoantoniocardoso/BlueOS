use std::process::ExitCode;

use catalog_provenance::provenance_anchor::{group_unanchored_reasons, migrate_catalog_sources};

fn main() -> ExitCode {
    let report = migrate_catalog_sources().expect("migrate catalog sources");
    println!("provenance_anchor_migrate");
    println!("sites touched: {}", report.sites_touched);
    println!("unanchored: {}", report.unanchored.len());
    for (reason, count) in group_unanchored_reasons(&report.unanchored) {
        println!("  {reason}: {count}");
    }
    ExitCode::SUCCESS
}
