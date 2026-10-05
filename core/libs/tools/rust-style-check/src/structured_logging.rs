use syn::{File, ItemMacro, StmtMacro, visit::Visit};

use crate::{Diagnostic, push};

const LOG_MACROS: &[&str] = &["error", "warn", "info", "debug", "trace"];

struct LoggingVisitor {
    diagnostics: Vec<Diagnostic>,
}

impl<'ast> Visit<'ast> for LoggingVisitor {
    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        if let Some(span) = log_message_violation_span(&node.mac.path, &node.mac.tokens) {
            push(
                &mut self.diagnostics,
                "structured_logging",
                span,
                "log message must be a constant string without format placeholders; use structured fields instead",
            );
        }
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_item_macro(&mut self, node: &'ast ItemMacro) {
        if let Some(span) = log_message_violation_span(&node.mac.path, &node.mac.tokens) {
            push(
                &mut self.diagnostics,
                "structured_logging",
                span,
                "log message must be a constant string without format placeholders; use structured fields instead",
            );
        }
        syn::visit::visit_item_macro(self, node);
    }

    fn visit_stmt_macro(&mut self, node: &'ast StmtMacro) {
        if let Some(span) = log_message_violation_span(&node.mac.path, &node.mac.tokens) {
            push(
                &mut self.diagnostics,
                "structured_logging",
                span,
                "log message must be a constant string without format placeholders; use structured fields instead",
            );
        }
        syn::visit::visit_stmt_macro(self, node);
    }
}

fn is_tracing_log_macro(path: &syn::Path) -> bool {
    let name = path
        .segments
        .last()
        .map(|segment| segment.ident.to_string());
    name.as_deref()
        .is_some_and(|name| LOG_MACROS.contains(&name))
}

fn log_message_violation_span(
    macro_path: &syn::Path,
    tokens: &proc_macro2::TokenStream,
) -> Option<proc_macro2::Span> {
    if !is_tracing_log_macro(macro_path) {
        return None;
    }
    walk_tokens_for_interpolation(tokens.clone())
}

fn walk_tokens_for_interpolation(tokens: proc_macro2::TokenStream) -> Option<proc_macro2::Span> {
    let mut pending = vec![tokens];
    while let Some(stream) = pending.pop() {
        for token in stream {
            match token {
                proc_macro2::TokenTree::Literal(literal) => {
                    let rendered = literal.to_string();
                    if rendered.starts_with('"') && message_has_placeholder(&rendered) {
                        return Some(literal.span());
                    }
                }
                proc_macro2::TokenTree::Group(group) => pending.push(group.stream()),
                _ => {}
            }
        }
    }
    None
}

fn message_has_placeholder(literal: &str) -> bool {
    let mut index = 0usize;
    let bytes = literal.as_bytes();
    while index < bytes.len() {
        if bytes[index] == b'{' {
            if bytes.get(index + 1) == Some(&b'{') {
                index += 2;
                continue;
            }
            return true;
        }
        index += 1;
    }
    false
}

pub(crate) fn check_file(syntax_tree: &File, diagnostics: &mut Vec<Diagnostic>) {
    let mut visitor = LoggingVisitor {
        diagnostics: Vec::new(),
    };
    visitor.visit_file(syntax_tree);
    diagnostics.extend(visitor.diagnostics);
}
