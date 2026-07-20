use std::process;

use blueos_catalog::tools::feature_provenance_export;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(err) = feature_provenance_export::run(&args) {
        eprintln!("export_feature_provenance: {err}");
        process::exit(1);
    }
}
