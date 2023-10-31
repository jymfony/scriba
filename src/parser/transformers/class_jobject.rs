use crate::parser::util::*;
use lazy_static::lazy_static;
use swc_common::util::take::Take;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

lazy_static! {
    static ref JOBJECT_ACCESSOR: MemberExpr = {
        let obj_expr = ident("__jymfony");
        let prop = ident("JObject");

        MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(obj_expr)),
            prop: MemberProp::Ident(prop),
        }
    };
}

pub fn class_jobject() -> impl VisitMut + Fold {
    as_folder(ClassJObject::default())
}

#[derive(Default)]
struct ClassJObject {}

impl VisitMut for ClassJObject {
    noop_visit_mut_type!();

    fn visit_mut_class(&mut self, n: &mut Class) {
        n.visit_mut_children_with(self);
        if n.super_class.is_some() {
            return;
        }

        n.super_class = Some(Box::new(Expr::Member(JOBJECT_ACCESSOR.clone())));
        for mut member in n.body.iter_mut() {
            if let ClassMember::Constructor(constructor) = &mut member {
                if let Some(block) = &mut constructor.body {
                    let call_expr = CallExpr {
                        span: DUMMY_SP,
                        callee: Callee::Super(Super::dummy()),
                        args: vec![],
                        type_args: None,
                    };

                    let call_super = Stmt::Expr(ExprStmt {
                        span: DUMMY_SP,
                        expr: Box::new(Expr::Call(call_expr)),
                    });

                    block.stmts.insert(0, call_super);
                }
            };
        }
    }
}
