use syn::{
    File, Item, ItemMod,
    visit::{Visit, visit_item_mod},
};

use crate::{
    Diagnostic,
    declaration_graph::{
        ItemKind, callees_in_function, cfg_is_test, classify_item, counts_toward_kind_order,
        impl_self_name, item_span, kind_label, kind_rank, module_positions,
        module_public_function_positions, public_function_names_in_module,
        public_type_dependency_graph, public_type_names_in_module, topological_order,
    },
    push,
};

struct OrderVisitor {
    diagnostics: Vec<Diagnostic>,
}

impl<'ast> Visit<'ast> for OrderVisitor {
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        if node.attrs.iter().any(cfg_is_test) {
            return;
        }
        if let Some((_, items)) = &node.content {
            self.check_module(items);
        }
        visit_item_mod(self, node);
    }
}

impl OrderVisitor {
    fn check_module(&mut self, items: &[Item]) {
        self.check_kind_order(items);
        self.check_type_order(items);
        self.check_impl_order(items);
        self.check_function_order(items);
    }

    fn check_kind_order(&mut self, items: &[Item]) {
        let mut previous: Option<ItemKind> = None;
        for item in items {
            if !counts_toward_kind_order(item) {
                continue;
            }
            let kind = classify_item(item);
            if kind == ItemKind::Other {
                continue;
            }
            if let Some(previous) = previous
                && kind_rank(kind) < kind_rank(previous)
            {
                push(
                    &mut self.diagnostics,
                    "declaration_order",
                    item_span(item),
                    format!(
                        "{} must not appear before {}",
                        kind_label(kind),
                        kind_label(previous)
                    ),
                );
            }
            previous = Some(kind);
        }
    }

    fn check_type_order(&mut self, items: &[Item]) {
        let type_names = public_type_names_in_module(items);
        if type_names.is_empty() {
            return;
        }
        let graph = public_type_dependency_graph(items, &type_names);
        let order = topological_order(&graph);
        let positions = module_positions(items, |item| match item {
            syn::Item::Struct(struct_item) => matches!(struct_item.vis, syn::Visibility::Public(_)),
            syn::Item::Enum(enum_item) => matches!(enum_item.vis, syn::Visibility::Public(_)),
            _ => false,
        });
        for (dependent, dependency) in order {
            let dependent_pos = positions.get(dependent);
            let dependency_pos = positions.get(dependency);
            if let (Some(dependent_pos), Some(dependency_pos)) = (dependent_pos, dependency_pos)
                && dependent_pos > dependency_pos
            {
                push(
                    &mut self.diagnostics,
                    "declaration_order",
                    item_span(&items[*dependent_pos]),
                    format!(
                        "type `{dependent}` must be declared before type `{dependency}`, which it uses"
                    ),
                );
            }
        }
    }

    fn check_impl_order(&mut self, items: &[Item]) {
        let mut by_type: alloc::collections::BTreeMap<String, Vec<(bool, usize)>> =
            alloc::collections::BTreeMap::new();
        for (index, item) in items.iter().enumerate() {
            let Item::Impl(impl_item) = item else {
                continue;
            };
            let type_name = impl_self_name(impl_item);
            if type_name.is_empty() {
                continue;
            }
            let is_trait = impl_item.trait_.is_some();
            by_type
                .entry(type_name)
                .or_default()
                .push((is_trait, index));
        }
        for (type_name, impls) in by_type {
            let mut seen_inherent = false;
            for (is_trait, index) in impls {
                if !is_trait {
                    seen_inherent = true;
                } else if seen_inherent {
                    let span = item_span(&items[index]);
                    push(
                        &mut self.diagnostics,
                        "declaration_order",
                        span,
                        format!(
                            "trait `impl` for `{type_name}` must come before the inherent `impl`"
                        ),
                    );
                }
            }
        }
    }

    fn check_function_order(&mut self, items: &[Item]) {
        let function_names = public_function_names_in_module(items);
        if function_names.len() < 2 {
            return;
        }
        let mut graph: alloc::collections::BTreeMap<String, alloc::collections::BTreeSet<String>> =
            alloc::collections::BTreeMap::new();
        for item in items {
            if let Item::Fn(function) = item
                && matches!(function.vis, syn::Visibility::Public(_))
            {
                let name = function.sig.ident.to_string();
                let callees = callees_in_function(function, &function_names);
                graph.insert(name, callees);
            }
        }
        let order = topological_order(&graph);
        let positions = module_public_function_positions(items);
        for (caller, callee) in order {
            let caller_pos = positions.get(caller);
            let callee_pos = positions.get(callee);
            if let (Some(caller_pos), Some(callee_pos)) = (caller_pos, callee_pos)
                && caller_pos > callee_pos
            {
                push(
                    &mut self.diagnostics,
                    "declaration_order",
                    item_span(&items[*caller_pos]),
                    format!("function `{caller}` must be declared before function `{callee}`"),
                );
            }
        }
    }
}

pub(crate) fn check_file(syntax_tree: &File, diagnostics: &mut Vec<Diagnostic>) {
    let mut visitor = OrderVisitor {
        diagnostics: Vec::new(),
    };
    visitor.check_module(&syntax_tree.items);
    visitor.visit_file(syntax_tree);
    diagnostics.extend(visitor.diagnostics);
}
