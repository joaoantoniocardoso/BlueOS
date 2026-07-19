use std::process;

use blueos_catalog::tools::feature_trace_enrich;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(err) = feature_trace_enrich::run(&args) {
        eprintln!("enrich_feature_traces: {err}");
        process::exit(1);
    }
}
