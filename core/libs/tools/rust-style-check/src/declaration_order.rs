use alloc::collections::{BTreeMap, BTreeSet};

use syn::{
    File, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, Visibility,
    spanned::Spanned,
    visit::{Visit, visit_item_mod},
};

use crate::{Diagnostic, push};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ItemKind {
    ConstOrAlias,
    Type,
    Impl,
    Function,
    TestModule,
    Other,
}

struct OrderVisitor {
    diagnostics: Vec<Diagnostic>,
}

struct TypeRefVisitor<'a> {
    known: &'a BTreeSet<String>,
    dependencies: &'a mut BTreeSet<String>,
}

struct CallVisitor<'a> {
    known: &'a BTreeSet<String>,
    callees: &'a mut BTreeSet<String>,
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
        let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for item in items {
            if let Item::Struct(struct_item) = item
                && matches!(struct_item.vis, Visibility::Public(_))
            {
                let name = struct_item.ident.to_string();
                let dependencies = type_dependencies_in_type(&struct_item.fields, &type_names);
                graph.insert(name, dependencies);
            }
            if let Item::Enum(enum_item) = item
                && matches!(enum_item.vis, Visibility::Public(_))
            {
                let name = enum_item.ident.to_string();
                let dependencies = enum_dependencies(enum_item, &type_names);
                graph.insert(name, dependencies);
            }
        }
        let order = topological_order(&graph);
        let positions = module_positions(items, |item| match item {
            Item::Struct(struct_item) => matches!(struct_item.vis, Visibility::Public(_)),
            Item::Enum(enum_item) => matches!(enum_item.vis, Visibility::Public(_)),
            _ => false,
        });
        for (dependent, dependency) in order {
            let dependent_pos = positions.get(&dependent);
            let dependency_pos = positions.get(&dependency);
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
        let mut by_type: BTreeMap<String, Vec<(bool, usize)>> = BTreeMap::new();
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
        let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for item in items {
            if let Item::Fn(function) = item
                && matches!(function.vis, Visibility::Public(_))
            {
                let name = function.sig.ident.to_string();
                let callees = callees_in_function(function, &function_names);
                graph.insert(name, callees);
            }
        }
        let order = topological_order(&graph);
        let positions = module_public_function_positions(items);
        for (caller, callee) in order {
            let caller_pos = positions.get(&caller);
            let callee_pos = positions.get(&callee);
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

impl<'ast> Visit<'ast> for TypeRefVisitor<'ast> {
    fn visit_path_segment(&mut self, segment: &'ast syn::PathSegment) {
        let name = segment.ident.to_string();
        if self.known.contains(&name) {
            self.dependencies.insert(name);
        }
        syn::visit::visit_path_segment(self, segment);
    }
}

impl<'ast> Visit<'ast> for CallVisitor<'ast> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func
            && path.qself.is_none()
            && path.path.segments.len() == 1
        {
            let name = path.path.segments[0].ident.to_string();
            if self.known.contains(&name) {
                self.callees.insert(name);
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

fn kind_rank(kind: ItemKind) -> u8 {
    match kind {
        ItemKind::ConstOrAlias => 0,
        ItemKind::Type => 1,
        ItemKind::Impl => 2,
        ItemKind::Function => 3,
        ItemKind::TestModule => 4,
        ItemKind::Other => 5,
    }
}

fn kind_label(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::ConstOrAlias => "constants and type aliases",
        ItemKind::Type => "type declarations",
        ItemKind::Impl => "`impl` blocks",
        ItemKind::Function => "free functions",
        ItemKind::TestModule => "`#[cfg(test)]` modules",
        ItemKind::Other => "other items",
    }
}

fn classify_item(item: &Item) -> ItemKind {
    match item {
        Item::Const(_) | Item::Static(_) | Item::Type(_) => ItemKind::ConstOrAlias,
        Item::Struct(_) | Item::Enum(_) | Item::Union(_) | Item::Trait(_) => ItemKind::Type,
        Item::Impl(_) => ItemKind::Impl,
        Item::Fn(_) => ItemKind::Function,
        Item::Mod(module) if module.attrs.iter().any(cfg_is_test) => ItemKind::TestModule,
        _ => ItemKind::Other,
    }
}

fn item_span(item: &Item) -> proc_macro2::Span {
    match item {
        Item::Const(item) => item.ident.span(),
        Item::Static(item) => item.ident.span(),
        Item::Type(item) => item.ident.span(),
        Item::Struct(item) => item.ident.span(),
        Item::Enum(item) => item.ident.span(),
        Item::Union(item) => item.ident.span(),
        Item::Trait(item) => item.ident.span(),
        Item::Impl(item) => item.self_ty.span(),
        Item::Fn(item) => item.sig.ident.span(),
        Item::Mod(item) => item.ident.span(),
        _ => proc_macro2::Span::call_site(),
    }
}

fn public_type_names_in_module(items: &[Item]) -> BTreeSet<String> {
    items
        .iter()
        .filter_map(|item| match item {
            Item::Struct(item) if matches!(item.vis, Visibility::Public(_)) => {
                Some(item.ident.to_string())
            }
            Item::Enum(item) if matches!(item.vis, Visibility::Public(_)) => {
                Some(item.ident.to_string())
            }
            Item::Union(item) if matches!(item.vis, Visibility::Public(_)) => {
                Some(item.ident.to_string())
            }
            _ => None,
        })
        .collect()
}

fn public_function_names_in_module(items: &[Item]) -> BTreeSet<String> {
    items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(function) if matches!(function.vis, Visibility::Public(_)) => {
                Some(function.sig.ident.to_string())
            }
            _ => None,
        })
        .collect()
}

fn counts_toward_kind_order(item: &Item) -> bool {
    match item {
        Item::Impl(_) => true,
        Item::Const(item) => matches!(item.vis, Visibility::Public(_)),
        Item::Static(item) => matches!(item.vis, Visibility::Public(_)),
        Item::Type(item) => matches!(item.vis, Visibility::Public(_)),
        Item::Struct(item) => matches!(item.vis, Visibility::Public(_)),
        Item::Enum(item) => matches!(item.vis, Visibility::Public(_)),
        Item::Union(item) => matches!(item.vis, Visibility::Public(_)),
        Item::Trait(item) => matches!(item.vis, Visibility::Public(_)),
        Item::Fn(function) => matches!(function.vis, Visibility::Public(_)),
        Item::Mod(module) => module.attrs.iter().any(cfg_is_test),
        _ => false,
    }
}

fn type_dependencies_in_type(fields: &syn::Fields, known: &BTreeSet<String>) -> BTreeSet<String> {
    let mut dependencies = BTreeSet::new();
    let mut visitor = TypeRefVisitor {
        known,
        dependencies: &mut dependencies,
    };
    visitor.visit_fields(fields);
    dependencies
}

fn enum_dependencies(enum_item: &ItemEnum, known: &BTreeSet<String>) -> BTreeSet<String> {
    let mut dependencies = BTreeSet::new();
    for variant in &enum_item.variants {
        dependencies.extend(type_dependencies_in_type(&variant.fields, known));
    }
    dependencies
}

fn callees_in_function(function: &ItemFn, known: &BTreeSet<String>) -> BTreeSet<String> {
    let mut callees = BTreeSet::new();
    let mut visitor = CallVisitor {
        known,
        callees: &mut callees,
    };
    visitor.visit_block(&function.block);
    callees
}

fn topological_order(graph: &BTreeMap<String, BTreeSet<String>>) -> Vec<(String, String)> {
    let mut edges = Vec::new();
    for (node, dependencies) in graph {
        for dependency in dependencies {
            edges.push((node.clone(), dependency.clone()));
        }
    }
    edges
}

fn module_positions<F>(items: &[Item], filter: F) -> BTreeMap<String, usize>
where
    F: Fn(&Item) -> bool,
{
    let mut positions = BTreeMap::new();
    for (index, item) in items.iter().enumerate() {
        if !filter(item) {
            continue;
        }
        let name = match item {
            Item::Struct(item) => item.ident.to_string(),
            Item::Enum(item) => item.ident.to_string(),
            _ => continue,
        };
        positions.insert(name, index);
    }
    positions
}

fn module_public_function_positions(items: &[Item]) -> BTreeMap<String, usize> {
    let mut positions = BTreeMap::new();
    for (index, item) in items.iter().enumerate() {
        if let Item::Fn(function) = item
            && matches!(function.vis, Visibility::Public(_))
        {
            positions.insert(function.sig.ident.to_string(), index);
        }
    }
    positions
}

fn impl_self_name(impl_item: &ItemImpl) -> String {
    match &*impl_item.self_ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn cfg_is_test(attribute: &syn::Attribute) -> bool {
    if !attribute.path().is_ident("cfg") {
        return false;
    }
    let syn::Meta::List(list) = &attribute.meta else {
        return false;
    };
    list.tokens.to_string().contains("test")
}

pub(crate) fn check_file(syntax_tree: &File, diagnostics: &mut Vec<Diagnostic>) {
    let mut visitor = OrderVisitor {
        diagnostics: Vec::new(),
    };
    visitor.check_module(&syntax_tree.items);
    visitor.visit_file(syntax_tree);
    diagnostics.extend(visitor.diagnostics);
}
