//! Prints every style violation in the workspace's Rust sources and exits non-zero when there is one.

mod workspace_scan;

use std::{env, path::PathBuf, process::ExitCode};

use blueos_rust_style_check::check_source;
use workspace_scan::{collect_rust_sources, print_diagnostic};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.is_empty() {
        eprintln!("usage: blueos-rust-style-check <workspace-root>");
        return ExitCode::from(2);
    }
    let workspace_root = PathBuf::from(&arguments[0]);
    let sources = collect_rust_sources(&workspace_root);
    let mut failed = false;
    for path in sources {
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("{}: {error}", path.display());
                failed = true;
                continue;
            }
        };
        let diagnostics = match check_source(&source, &path) {
            Ok(diagnostics) => diagnostics,
            Err(message) => {
                eprintln!("{message}");
                failed = true;
                continue;
            }
        };
        for diagnostic in diagnostics {
            print_diagnostic(&path, &diagnostic);
            failed = true;
        }
    }
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
