use std::process;

use blueos_catalog::tools::feature_trace_report;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(err) = feature_trace_report::run(&args) {
        eprintln!("feature_trace_report: {err}");
        process::exit(1);
    }
}
