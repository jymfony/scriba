#[cfg(not(test))]
use rand::prelude::*;
use swc_ecma_ast::*;
use swc_ecma_utils::private_ident;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

pub fn anonymous_expr() -> impl VisitMut + Fold {
    as_folder(AnonymousExpr::default())
}

#[derive(Default)]
struct AnonymousExpr {
    #[cfg(test)]
    test_current_id: i32,
}

impl AnonymousExpr {
    fn gen_anonymous_ident(&mut self) -> Ident {
        #[cfg(not(test))]
        let rnd = thread_rng().gen_range(0..1000000);
        #[cfg(test)]
        let rnd = {
            self.test_current_id += 1;
            self.test_current_id
        };

        let ident = format!("_anonymous_xΞ{:X}", rnd);

        private_ident!(ident)
    }
}

impl VisitMut for AnonymousExpr {
    noop_visit_mut_type!();

    fn visit_mut_class_expr(&mut self, n: &mut ClassExpr) {
        n.visit_mut_children_with(self);

        if n.ident.is_none() {
            n.ident = Some(self.gen_anonymous_ident());
        }
    }

    fn visit_mut_fn_expr(&mut self, n: &mut FnExpr) {
        n.visit_mut_children_with(self);

        if n.ident.is_none() {
            n.ident = Some(self.gen_anonymous_ident());
        }
    }
}
