use syn::{
    Expr, ExprAsync, ExprCall, ExprPath, File, Pat, Stmt,
    spanned::Spanned,
    visit::{Visit, visit_block, visit_expr_path},
};

use crate::{Diagnostic, push};

struct SpawnVisitor {
    diagnostics: Vec<Diagnostic>,
}

impl SpawnVisitor {
    fn check_block(&mut self, statements: &[Stmt]) {
        for (index, statement) in statements.iter().enumerate() {
            let Stmt::Expr(expression, _) = statement else {
                continue;
            };
            let Expr::Call(call) = expression else {
                continue;
            };
            if !is_spawn_call(&call.func) {
                continue;
            }
            let Some(argument) = call.args.first() else {
                continue;
            };
            match argument {
                Expr::Async(async_block) if async_block.capture.is_some() => {
                    let prior = &statements[..index];
                    if clone_binding_before_spawn(prior, async_block) {
                        push(
                            &mut self.diagnostics,
                            "clone_before_spawn",
                            async_block.span(),
                            "bind values cloned for `async move` inside a block attached to `spawn`, not in the enclosing scope",
                        );
                    }
                }
                Expr::Block(block) => {
                    self.check_block(&block.block.stmts);
                }
                _ => {}
            }
        }
    }
}

fn clone_binding_before_spawn(prior: &[Stmt], async_block: &ExprAsync) -> bool {
    let captured = identifiers_in_async_block(async_block);
    for statement in prior {
        if let Some(name) = clone_binding_name(statement)
            && captured.contains(&name)
        {
            return true;
        }
    }
    false
}

fn identifiers_in_async_block(async_block: &ExprAsync) -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    let mut visitor = CaptureVisitor { names: &mut names };
    visitor.visit_block(&async_block.block);
    names
}

struct CaptureVisitor<'a> {
    names: &'a mut std::collections::BTreeSet<String>,
}

impl<'ast> Visit<'ast> for CaptureVisitor<'ast> {
    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        if node.qself.is_none() && node.path.segments.len() == 1 {
            self.names.insert(node.path.segments[0].ident.to_string());
        }
        visit_expr_path(self, node);
    }
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

pub fn check_file(syntax_tree: &File, diagnostics: &mut Vec<Diagnostic>) {
    let mut visitor = SpawnVisitor {
        diagnostics: Vec::new(),
    };
    for item in &syntax_tree.items {
        if let syn::Item::Fn(function) = item {
            visitor.check_block(&function.block.stmts);
        }
        if let syn::Item::Mod(module) = item {
            if let Some((_, items)) = &module.content {
                for inner in items {
                    if let syn::Item::Fn(function) = inner {
                        visitor.check_block(&function.block.stmts);
                    }
                }
            }
        }
    }
    visitor.visit_file(syntax_tree);
    diagnostics.extend(visitor.diagnostics);
}
