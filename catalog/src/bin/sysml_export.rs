use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use blueos_catalog::export_sysml::{export_sysml, SysmlExportFilter, DEFAULT_SYSML_SUBSET_GOLDEN};
use blueos_catalog::Catalog;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }

    let check = args.iter().any(|arg| arg == "--check" || arg == "--diff");
    let snapshot = args.iter().any(|arg| arg == "--snapshot");
    let full = args.iter().any(|arg| arg == "--full");
    let output_path = flag_value(&args, "--output").map(PathBuf::from);
    let golden_path = flag_value(&args, "--golden")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_SYSML_SUBSET_GOLDEN)
        });

    let catalog = Catalog::bootstrap();
    let filter = if full {
        None
    } else {
        Some(SysmlExportFilter::subset_golden())
    };
    let output = export_sysml(&catalog, filter.as_ref());

    if snapshot {
        return write_output(&golden_path, &output);
    }

    if check {
        let baseline = match fs::read_to_string(&golden_path) {
            Ok(baseline) => baseline,
            Err(error) => {
                eprintln!(
                    "sysml_export compare failed: {error} (run with --snapshot to create {})",
                    golden_path.display()
                );
                return ExitCode::FAILURE;
            }
        };
        if baseline != output {
            eprintln!(
                "sysml_export: output differs from {}",
                golden_path.display()
            );
            eprintln!("run: cargo run -q --bin sysml_export -- --snapshot");
            return ExitCode::FAILURE;
        }
        println!("sysml_export: matches {}", golden_path.display());
        return ExitCode::SUCCESS;
    }

    if let Some(path) = output_path {
        return write_output(&path, &output);
    }

    print!("{output}");
    ExitCode::SUCCESS
}

fn write_output(path: &Path, output: &str) -> ExitCode {
    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("sysml_export write failed: {error}");
            return ExitCode::FAILURE;
        }
    }
    match fs::write(path, output) {
        Ok(()) => {
            println!("wrote {}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("sysml_export write failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn print_help() {
    eprintln!(
        "usage: sysml_export [--check|--snapshot] [--full] [--output PATH] [--golden PATH]\n\
         \n\
         Exports the catalog as SysML v2 textual notation.\n\
         Default mode prints the golden subset to stdout.\n\
         --check compares against the committed golden (gate.sh).\n\
         --snapshot rewrites the golden file.\n\
         --full exports the entire catalog (not gated)."
    );
}
