use std::path::PathBuf;
use std::process;

use catalog_analysis::journey_matrix::{
    blank_both_violations, build_journey_matrix, format_matrix, load_report_hits,
};

use blueos_catalog::catalog::Catalog;

fn main() {
    let mut merge_paths: Vec<PathBuf> = Vec::new();
    let args: Vec<String> = std::env::args().collect();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--merge-report" => {
                index += 1;
                let Some(path) = args.get(index) else {
                    eprintln!("--merge-report requires a file or directory");
                    process::exit(2);
                };
                merge_paths.push(PathBuf::from(path));
            }
            "-h" | "--help" => {
                eprintln!(
                    "Usage: journey_matrix [--merge-report <file-or-dir>]...\n\
                     Two-cell (backend/UI) coverage matrix. derive_automatable is a hint only."
                );
                process::exit(0);
            }
            other => {
                eprintln!("unknown argument: {other}");
                process::exit(2);
            }
        }
        index += 1;
    }

    let hits = match load_report_hits(&merge_paths) {
        Ok(hits) => hits,
        Err(err) => {
            eprintln!("{err}");
            process::exit(2);
        }
    };
    let catalog = Catalog::bootstrap();
    let matrix = build_journey_matrix(&catalog, &hits);
    print!("{}", format_matrix(&matrix));
    let blanks = blank_both_violations(&matrix);
    if !blanks.is_empty() {
        process::exit(1);
    }
}
