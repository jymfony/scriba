use crate::parser::util::ident;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

pub fn class_reflection_decorators() -> impl VisitMut + Fold {
    as_folder(ClassReflectionDecorators::default())
}

#[derive(Default)]
struct ClassReflectionDecorators {}

fn visit_mut_class(n: &mut Class) {
    let reflect_ident = Expr::Ident(ident("__jymfony_reflect"));
    for member in n.body.iter_mut() {
        match member {
            ClassMember::Method(m) => {
                m.function.decorators.push(Decorator {
                    span: DUMMY_SP,
                    expr: Box::new(reflect_ident.clone()),
                });
            }
            ClassMember::PrivateMethod(m) => {
                m.function.decorators.push(Decorator {
                    span: DUMMY_SP,
                    expr: Box::new(reflect_ident.clone()),
                });
            }
            ClassMember::ClassProp(p) => {
                p.decorators.push(Decorator {
                    span: DUMMY_SP,
                    expr: Box::new(reflect_ident.clone()),
                });
            }
            ClassMember::PrivateProp(p) => {
                p.decorators.push(Decorator {
                    span: DUMMY_SP,
                    expr: Box::new(reflect_ident.clone()),
                });
            }
            ClassMember::AutoAccessor(a) => {
                a.decorators.push(Decorator {
                    span: DUMMY_SP,
                    expr: Box::new(reflect_ident.clone()),
                });
            }
            _ => { // Do nothing }
            }
        }
    }
}

impl VisitMut for ClassReflectionDecorators {
    noop_visit_mut_type!();

    fn visit_mut_class_decl(&mut self, n: &mut ClassDecl) {
        visit_mut_class(&mut n.class)
    }

    fn visit_mut_class_expr(&mut self, n: &mut ClassExpr) {
        visit_mut_class(&mut n.class)
    }
}
