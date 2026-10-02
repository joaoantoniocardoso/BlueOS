use syn::{
    Attribute, File,
    spanned::Spanned,
    visit::{Visit, visit_attribute},
};

use crate::{Diagnostic, push};

struct AllowVisitor<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'ast> Visit<'ast> for AllowVisitor<'ast> {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if attribute.path().is_ident("allow") {
            push(
                self.diagnostics,
                "allow_attributes",
                attribute.span(),
                "use #[expect(lint, reason = \"...\")] instead of #[allow]",
            );
        }
        visit_attribute(self, attribute);
    }
}

pub fn check_file(syntax_tree: &File, diagnostics: &mut Vec<Diagnostic>) {
    let mut visitor = AllowVisitor { diagnostics };
    visitor.visit_file(syntax_tree);
}
