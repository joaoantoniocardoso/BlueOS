use alloc::collections::BTreeSet;

use syn::{
    Expr, ExprAsync, ExprCall, ExprPath, File, Pat, Stmt,
    spanned::Spanned,
    visit::{Visit, visit_block, visit_expr_path},
};

use crate::{Diagnostic, push};

struct SpawnVisitor {
    diagnostics: Vec<Diagnostic>,
}

struct CaptureVisitor<'a> {
    names: &'a mut BTreeSet<String>,
}

impl<'ast> Visit<'ast> for SpawnVisitor {
    fn visit_expr_block(&mut self, node: &'ast syn::ExprBlock) {
        self.check_block(&node.block.stmts);
        visit_block(self, &node.block);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_block(&node.block.stmts);
        syn::visit::visit_item_fn(self, node);
    }
}

impl SpawnVisitor {
    fn check_block(&mut self, statements: &[Stmt]) {
        for span in spawn_async_move_clone_spans(statements) {
            push(
                &mut self.diagnostics,
                "clone_before_spawn",
                span,
                "bind values cloned for `async move` inside a block attached to `spawn`, not in the enclosing scope",
            );
        }
    }
}

impl<'ast> Visit<'ast> for CaptureVisitor<'ast> {
    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        if node.qself.is_none() && node.path.segments.len() == 1 {
            self.names.insert(node.path.segments[0].ident.to_string());
        }
        visit_expr_path(self, node);
    }
}

fn spawn_async_move_clone_spans(
    statements: &[Stmt],
) -> impl Iterator<Item = proc_macro2::Span> + '_ {
    statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            let Stmt::Expr(expression, _) = statement else {
                return None;
            };
            let Expr::Call(call) = expression else {
                return None;
            };
            if !is_spawn_call(&call.func) {
                return None;
            }
            let Expr::Async(async_block) = call.args.first()? else {
                return None;
            };
            async_block.capture?;
            let prior = &statements[..index];
            clone_binding_before_spawn(prior, async_block).then_some(async_block.span())
        })
}

fn clone_binding_before_spawn(prior: &[Stmt], async_block: &ExprAsync) -> bool {
    let captured = identifiers_in_async_block(async_block);
    prior
        .iter()
        .any(|statement| clone_binding_name(statement).is_some_and(|name| captured.contains(&name)))
}

fn identifiers_in_async_block(async_block: &ExprAsync) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut visitor = CaptureVisitor { names: &mut names };
    visitor.visit_block(&async_block.block);
    names
}

fn clone_binding_name(statement: &Stmt) -> Option<String> {
    let Stmt::Local(local) = statement else {
        return None;
    };
    let Pat::Ident(pattern) = &local.pat else {
        return None;
    };
    let name = pattern.ident.to_string();
    if expression_clones(
        local
            .init
            .as_ref()
            .map(|initializer| initializer.expr.as_ref()),
    ) {
        Some(name)
    } else {
        None
    }
}

fn expression_clones(expression: Option<&Expr>) -> bool {
    match expression {
        Some(Expr::MethodCall(method)) => method.method == "clone",
        Some(Expr::Call(call)) => is_arc_clone_call(call),
        _ => false,
    }
}

fn is_arc_clone_call(call: &ExprCall) -> bool {
    match &*call.func {
        Expr::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "clone"),
        _ => false,
    }
}

fn is_spawn_call(expression: &Expr) -> bool {
    match expression {
        Expr::Path(ExprPath { path, .. }) => path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "spawn"),
        _ => false,
    }
}

fn seed_spawn_checks(syntax_tree: &File, visitor: &mut SpawnVisitor) {
    for item in &syntax_tree.items {
        if let syn::Item::Fn(function) = item {
            visitor.check_block(&function.block.stmts);
        }
        if let syn::Item::Mod(module) = item
            && let Some((_, items)) = &module.content
        {
            for inner in items {
                if let syn::Item::Fn(function) = inner {
                    visitor.check_block(&function.block.stmts);
                }
            }
        }
    }
}

pub(crate) fn check_file(syntax_tree: &File, diagnostics: &mut Vec<Diagnostic>) {
    let mut visitor = SpawnVisitor {
        diagnostics: Vec::new(),
    };
    seed_spawn_checks(syntax_tree, &mut visitor);
    visitor.visit_file(syntax_tree);
    diagnostics.extend(visitor.diagnostics);
}
