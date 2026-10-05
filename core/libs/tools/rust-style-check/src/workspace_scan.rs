//! Walks a workspace tree and prints [`Diagnostic`] lines for the CLI.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use blueos_rust_style_check::Diagnostic;

pub(crate) fn print_diagnostic(path: &Path, diagnostic: &Diagnostic) {
    eprintln!(
        "{}:{}: {}: {}",
        path.display(),
        diagnostic.line,
        diagnostic.rule,
        diagnostic.message
    );
}

pub(crate) fn collect_rust_sources(workspace_root: &Path) -> Vec<PathBuf> {
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
        if path
            .components()
            .any(|component| component.as_os_str() == "fixtures")
        {
            continue;
        }
        paths.push(path);
    }
    paths.sort();
    paths
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use blueos_rust_style_check::Diagnostic;

    use super::{collect_rust_sources, print_diagnostic};

    #[test]
    fn collect_rust_sources_skips_target_and_fixtures() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let paths = collect_rust_sources(root);
        assert!(paths.iter().any(|path| path.ends_with("src/lib.rs")));
        assert!(paths.iter().all(|path| {
            !path
                .components()
                .any(|component| component.as_os_str() == "target")
        }));
        assert!(paths.iter().all(|path| {
            !path
                .components()
                .any(|component| component.as_os_str() == "fixtures")
        }));
    }

    #[test]
    fn print_diagnostic_formats_a_line() {
        let diagnostic = Diagnostic {
            rule: "declaration_order",
            line: 3,
            message: "example".into(),
        };
        print_diagnostic(Path::new("example.rs"), &diagnostic);
    }
}
