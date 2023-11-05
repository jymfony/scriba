use crate::parser::util::ident;
use swc_common::util::take::Take;
use swc_ecma_ast::*;
use swc_ecma_utils::ExprFactory;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

pub fn lazy_object_construction() -> impl VisitMut + Fold {
    as_folder(LazyObjectConstruction::default())
}

#[derive(Default)]
struct LazyObjectConstruction {}

impl VisitMut for LazyObjectConstruction {
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, n: &mut Expr) {
        n.visit_mut_children_with(self);

        let Expr::New(new_expr) = n else {
            return;
        };
        let NewExpr {
            span,
            callee,
            args,
            type_args,
        } = new_expr.take();

        let new_args = vec![vec![callee.as_arg()], args.unwrap_or_default()]
            .into_iter()
            .flatten()
            .collect();

        *n = Expr::Call(CallExpr {
            span,
            callee: Callee::Expr(Box::new(Expr::Ident(ident("_construct_jobject")))),
            args: new_args,
            type_args,
        });
    }
}
