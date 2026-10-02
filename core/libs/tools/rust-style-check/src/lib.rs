//! Syn-based linter for BlueOS Rust style rules that `clippy` cannot enforce (D-30).

mod allow_attributes;
mod clone_before_spawn;
mod declaration_order;
mod import_groups;
mod structured_logging;

use std::path::Path;

use proc_macro2::Span;
use syn::File;

/// One style violation with a stable rule name for tests and CI output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub rule: &'static str,
    pub line: usize,
    pub message: String,
}

/// Parses `source` and returns every violation. `path` is used only for error messages.
pub fn check_source(source: &str, path: &Path) -> Result<Vec<Diagnostic>, String> {
    let syntax_tree: File = syn::parse_file(source)
        .map_err(|error| format!("{}: parse error: {error}", path.display()))?;
    Ok(collect_diagnostics(&syntax_tree, path))
}

fn collect_diagnostics(syntax_tree: &File, path: &Path) -> Vec<Diagnostic> {
    // Generated IDL message files declare helper enums before the struct that uses them; the type-order
    // rule expects the opposite, so checking declaration order there would fight the codegen layout.
    let skip_declaration_order = path
        .components()
        .any(|component| component.as_os_str() == "generated");
    let mut diagnostics = Vec::new();
    allow_attributes::check_file(syntax_tree, &mut diagnostics);
    import_groups::check_file(syntax_tree, &mut diagnostics);
    if !skip_declaration_order {
        declaration_order::check_file(syntax_tree, &mut diagnostics);
    }
    structured_logging::check_file(syntax_tree, &mut diagnostics);
    clone_before_spawn::check_file(syntax_tree, &mut diagnostics);
    diagnostics.sort_by_key(|diagnostic| (diagnostic.line, diagnostic.rule));
    diagnostics
}

pub(crate) fn push(
    diagnostics: &mut Vec<Diagnostic>,
    rule: &'static str,
    span: Span,
    message: impl Into<String>,
) {
    diagnostics.push(Diagnostic {
        rule,
        line: line_of(span),
        message: message.into(),
    });
}

pub(crate) fn line_of(span: Span) -> usize {
    span.start().line
}
