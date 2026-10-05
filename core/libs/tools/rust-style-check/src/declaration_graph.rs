//! Graph and classification helpers for declaration-order checking.

use alloc::collections::{BTreeMap, BTreeSet};

use syn::{
    Item, ItemEnum, ItemFn, ItemImpl, Visibility,
    spanned::Spanned,
    visit::{Visit, visit_expr_call, visit_path_segment},
};

const KIND_RANK_IMPL: u8 = 2;
const KIND_RANK_FUNCTION: u8 = 3;
const KIND_RANK_TEST_MODULE: u8 = 4;
const KIND_RANK_OTHER: u8 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ItemKind {
    ConstOrAlias,
    Type,
    Impl,
    Function,
    TestModule,
    Other,
}

pub(crate) struct TypeRefVisitor<'a> {
    known: &'a BTreeSet<String>,
    dependencies: &'a mut BTreeSet<String>,
}

pub(crate) struct CallVisitor<'a> {
    known: &'a BTreeSet<String>,
    callees: &'a mut BTreeSet<String>,
}

impl<'ast> Visit<'ast> for TypeRefVisitor<'ast> {
    fn visit_path_segment(&mut self, segment: &'ast syn::PathSegment) {
        let name = segment.ident.to_string();
        if self.known.contains(&name) {
            self.dependencies.insert(name);
        }
        visit_path_segment(self, segment);
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
        visit_expr_call(self, node);
    }
}

pub(crate) fn public_type_dependency_graph(
    items: &[Item],
    type_names: &BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for item in items {
        if let Item::Struct(struct_item) = item
            && matches!(struct_item.vis, Visibility::Public(_))
        {
            let name = struct_item.ident.to_string();
            let dependencies = type_dependencies_in_type(&struct_item.fields, type_names);
            graph.insert(name, dependencies);
        }
        if let Item::Enum(enum_item) = item
            && matches!(enum_item.vis, Visibility::Public(_))
        {
            let name = enum_item.ident.to_string();
            let dependencies = enum_dependencies(enum_item, type_names);
            graph.insert(name, dependencies);
        }
    }
    graph
}

pub(crate) fn kind_rank(kind: ItemKind) -> u8 {
    match kind {
        ItemKind::ConstOrAlias => 0,
        ItemKind::Type => 1,
        ItemKind::Impl => KIND_RANK_IMPL,
        ItemKind::Function => KIND_RANK_FUNCTION,
        ItemKind::TestModule => KIND_RANK_TEST_MODULE,
        ItemKind::Other => KIND_RANK_OTHER,
    }
}

pub(crate) fn kind_label(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::ConstOrAlias => "constants and type aliases",
        ItemKind::Type => "type declarations",
        ItemKind::Impl => "`impl` blocks",
        ItemKind::Function => "free functions",
        ItemKind::TestModule => "`#[cfg(test)]` modules",
        ItemKind::Other => "other items",
    }
}

pub(crate) fn classify_item(item: &Item) -> ItemKind {
    match item {
        Item::Const(_) | Item::Static(_) | Item::Type(_) => ItemKind::ConstOrAlias,
        Item::Struct(_) | Item::Enum(_) | Item::Union(_) | Item::Trait(_) => ItemKind::Type,
        Item::Impl(_) => ItemKind::Impl,
        Item::Fn(_) => ItemKind::Function,
        Item::Mod(module) if module.attrs.iter().any(cfg_is_test) => ItemKind::TestModule,
        _ => ItemKind::Other,
    }
}

pub(crate) fn item_span(item: &Item) -> proc_macro2::Span {
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

pub(crate) fn public_type_names_in_module(items: &[Item]) -> BTreeSet<String> {
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

pub(crate) fn public_function_names_in_module(items: &[Item]) -> BTreeSet<String> {
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

pub(crate) fn counts_toward_kind_order(item: &Item) -> bool {
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

pub(crate) fn callees_in_function(function: &ItemFn, known: &BTreeSet<String>) -> BTreeSet<String> {
    let mut callees = BTreeSet::new();
    let mut visitor = CallVisitor {
        known,
        callees: &mut callees,
    };
    visitor.visit_block(&function.block);
    callees
}

pub(crate) fn topological_order(graph: &BTreeMap<String, BTreeSet<String>>) -> Vec<(&str, &str)> {
    let mut edges = Vec::new();
    for (node, dependencies) in graph {
        for dependency in dependencies {
            edges.push((node.as_str(), dependency.as_str()));
        }
    }
    edges
}

pub(crate) fn module_positions<F>(items: &[Item], filter: F) -> BTreeMap<String, usize>
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

pub(crate) fn module_public_function_positions(items: &[Item]) -> BTreeMap<String, usize> {
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

pub(crate) fn impl_self_name(impl_item: &ItemImpl) -> String {
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

pub(crate) fn cfg_is_test(attribute: &syn::Attribute) -> bool {
    if !attribute.path().is_ident("cfg") {
        return false;
    }
    let syn::Meta::List(list) = &attribute.meta else {
        return false;
    };
    list.tokens.to_string().contains("test")
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
