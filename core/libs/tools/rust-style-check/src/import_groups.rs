use syn::{
    File, Item, ItemMod, ItemUse, UseTree,
    spanned::Spanned,
    visit::{Visit, visit_item_mod},
};

use crate::{Diagnostic, push};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
enum ImportGroup {
    Std,
    ThirdParty,
    Blueos,
    Owned,
    Relative,
}

fn group_label(group: ImportGroup) -> &'static str {
    match group {
        ImportGroup::Std => "std",
        ImportGroup::ThirdParty => "third-party",
        ImportGroup::Blueos => "blueos",
        ImportGroup::Owned => "owned (`crate::`)",
        ImportGroup::Relative => "relative (`self::` / `super::`)",
    }
}

impl ImportGroup {
    fn from_crate_root(root: &str) -> Self {
        match root {
            "std" | "core" | "alloc" => ImportGroup::Std,
            "crate" => ImportGroup::Owned,
            "self" | "super" => ImportGroup::Relative,
            _ if root.starts_with("blueos") => ImportGroup::Blueos,
            _ => ImportGroup::ThirdParty,
        }
    }
}

struct ImportVisitor {
    diagnostics: Vec<Diagnostic>,
}

impl ImportVisitor {
    fn check_module(&mut self, items: &[Item]) {
        let mut index = 0usize;
        while index < items.len() {
            if !matches!(items[index], Item::Use(_)) {
                break;
            }
            index += 1;
        }
        if index == 0 {
            return;
        }
        let import_block = &items[..index];
        self.check_import_block(import_block);
    }

    fn check_import_block(&mut self, items: &[Item]) {
        let mut previous_group: Option<ImportGroup> = None;
        let mut previous_line: Option<usize> = None;
        let mut crate_roots: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();

        for item in items {
            let Item::Use(use_item) = item else {
                continue;
            };
            let group = import_group_for_use(use_item);
            let line = crate::line_of(use_item.span());
            if let Some(previous_group) = previous_group
                && group != previous_group
                && previous_line == Some(line - 1)
            {
                push(
                    &mut self.diagnostics,
                    "import_groups",
                    use_item.span(),
                    format!(
                        "expected a blank line between {} and {} import groups",
                        group_label(previous_group),
                        group_label(group)
                    ),
                );
            }
            if let Some(previous_group) = previous_group
                && group < previous_group
            {
                push(
                    &mut self.diagnostics,
                    "import_groups",
                    use_item.span(),
                    format!(
                        "import group {} must not appear before {}",
                        group_label(group),
                        group_label(previous_group)
                    ),
                );
            }
            previous_group = Some(group);
            previous_line = Some(line);

            let roots = crate_roots_in_use(use_item);
            for root in roots {
                if let Some(first_line) = crate_roots.get(&root)
                    && *first_line != line
                {
                    push(
                        &mut self.diagnostics,
                        "import_chaining",
                        use_item.span(),
                        format!("crate `{root}` must be imported in one chained `use` statement"),
                    );
                } else {
                    crate_roots.entry(root).or_insert(line);
                }
            }
        }
    }
}

fn import_group_for_use(use_item: &ItemUse) -> ImportGroup {
    let root = first_crate_root(&use_item.tree);
    ImportGroup::from_crate_root(&root)
}

fn first_crate_root(tree: &UseTree) -> String {
    match tree {
        UseTree::Path(use_path) => use_path.ident.to_string(),
        UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) | UseTree::Group(_) => {
            String::new()
        }
    }
}

fn crate_roots_in_use(use_item: &ItemUse) -> Vec<String> {
    let root = first_crate_root(&use_item.tree);
    if root.is_empty() {
        Vec::new()
    } else {
        vec![root]
    }
}

impl<'ast> Visit<'ast> for ImportVisitor {
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        if let Some((_, items)) = &node.content {
            self.check_module(items);
        }
        visit_item_mod(self, node);
    }
}

pub fn check_file(syntax_tree: &File, diagnostics: &mut Vec<Diagnostic>) {
    let mut visitor = ImportVisitor {
        diagnostics: Vec::new(),
    };
    visitor.check_module(&syntax_tree.items);
    visitor.visit_file(syntax_tree);
    diagnostics.extend(visitor.diagnostics);
}
