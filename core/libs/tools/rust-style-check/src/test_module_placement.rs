use syn::{
    File, Ident, ItemMod,
    spanned::Spanned,
    visit::{Visit, visit_item_mod},
};

use crate::{Diagnostic, push};

struct TestModuleVisitor<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'ast> Visit<'ast> for TestModuleVisitor<'ast> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let is_test_only = module.attrs.iter().any(|attribute| {
            attribute.path().is_ident("cfg")
                && attribute
                    .parse_args::<Ident>()
                    .is_ok_and(|argument| argument == "test")
        });
        if is_test_only && module.content.is_none() {
            push(
                self.diagnostics,
                "test_module_placement",
                module.span(),
                "write #[cfg(test)] mod tests { ... } inline for tests of private items, and move tests of the \
                 public API to tests/",
            );
        }
        visit_item_mod(self, module);
    }
}

pub(crate) fn check_file(syntax_tree: &File, diagnostics: &mut Vec<Diagnostic>) {
    let mut visitor = TestModuleVisitor { diagnostics };
    visitor.visit_file(syntax_tree);
}
