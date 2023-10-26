use std::mem::replace;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

pub fn remove_assert_calls() -> impl VisitMut + Fold {
    as_folder(RemoveAssertCalls::default())
}

#[derive(Default)]
struct RemoveAssertCalls {}

impl VisitMut for RemoveAssertCalls {
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, n: &mut Expr) {
        let Expr::Call(call) = n else {
            return;
        };
        let Callee::Expr(expr) = &mut call.callee else {
            return;
        };
        let Expr::Ident(id) = expr.as_ref() else {
            return;
        };

        if id.sym == "__assert" {
            let _ = replace(
                n,
                Expr::Unary(UnaryExpr {
                    span: DUMMY_SP,
                    op: UnaryOp::Void,
                    arg: Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: 0.0,
                        raw: Some("0".into()),
                    }))),
                }),
            );
        }
    }
}
