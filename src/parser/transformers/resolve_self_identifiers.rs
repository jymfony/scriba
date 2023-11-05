use std::mem::replace;
use swc_common::{Mark, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

pub fn resolve_self_identifiers(unresolved_mark: Mark) -> impl VisitMut + Fold {
    as_folder(ResolveSelfIdentifiers {
        unresolved: SyntaxContext::empty().apply_mark(unresolved_mark),
        class_stack: vec![],
    })
}

#[derive(Default)]
struct ResolveSelfIdentifiers {
    unresolved: SyntaxContext,
    class_stack: Vec<Ident>,
}

impl VisitMut for ResolveSelfIdentifiers {
    noop_visit_mut_type!();

    fn visit_mut_class_decl(&mut self, n: &mut ClassDecl) {
        self.class_stack.push(n.ident.clone());
        n.visit_mut_children_with(self);
        let _ = self.class_stack.pop();
    }

    fn visit_mut_class_expr(&mut self, n: &mut ClassExpr) {
        self.class_stack.push(
            n.ident
                .clone()
                .expect("anonymous_expr transformer needs to be called before this"),
        );
        n.visit_mut_children_with(self);
        let _ = self.class_stack.pop();
    }

    fn visit_mut_ident(&mut self, n: &mut Ident) {
        if n.span.ctxt == self.unresolved && n.sym == "__self" {
            let Some(current_class) = self.class_stack.last() else {
                return;
            };
            let _ = replace(n, current_class.clone());
        }
    }
}
