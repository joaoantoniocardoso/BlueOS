use std::{
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};

use walkdir::WalkDir;

use blueos_rust_style_check::{Diagnostic, check_source};

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

fn print_diagnostic(path: &Path, diagnostic: &Diagnostic) {
    eprintln!(
        "{}:{}: {}: {}",
        path.display(),
        diagnostic.line,
        diagnostic.rule,
        diagnostic.message
    );
}

fn collect_rust_sources(workspace_root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(workspace_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.into_path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        if path
            .components()
            .any(|component| component.as_os_str() == "target")
        {
            continue;
        }
        if path.components().any(|component| {
            let name = component.as_os_str();
            name == "fixtures" || name == "generated"
        }) {
            continue;
        }
        paths.push(path);
    }
    paths.sort();
    paths
}
