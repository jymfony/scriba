use crate::generate_uuid;
use crate::parser::util::ident;
use crate::reflection::{register_class, ReflectionData};
use std::collections::HashMap;
use std::rc::Rc;
use swc_common::comments::{CommentKind, Comments};
use swc_common::{Span, Spanned, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_utils::ExprFactory;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

pub fn class_reflection_decorators<'a, C: Comments + 'a>(
    filename: Option<&'a str>,
    namespace: Option<&'a str>,
    comments: Rc<C>,
) -> impl VisitMut + Fold + 'a {
    as_folder(ClassReflectionDecorators {
        filename,
        namespace,
        comments,
    })
}

struct ClassReflectionDecorators<'a, C: Comments> {
    filename: Option<&'a str>,
    namespace: Option<&'a str>,
    comments: Rc<C>,
}

impl<'a, C: Comments> ClassReflectionDecorators<'a, C> {
    fn get_element_docblock(&self, span: Span) -> Option<String> {
        self.comments
            .get_leading(span.lo)
            .iter()
            .flatten()
            .rev()
            .find_map(|cmt| {
                if cmt.kind == CommentKind::Block && cmt.text.starts_with("*") {
                    Some(format!("/*{}*/", cmt.text))
                } else {
                    None
                }
            })
    }

    fn process_class(&self, n: &mut Class, name: Ident) {
        let id = generate_uuid();
        let mut docblock = HashMap::new();
        if n.span != DUMMY_SP {
            docblock.insert(n.span, self.get_element_docblock(n.span));
        }

        n.decorators.push(Decorator {
            span: DUMMY_SP,
            expr: Box::new(Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: ident("__jymfony_reflect").as_callee(),
                args: vec![id.to_string().as_arg()],
                type_args: None,
            })),
        });

        for (idx, member) in n.body.iter_mut().enumerate() {
            let reflect_ident = Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: ident("__jymfony_reflect").as_callee(),
                args: vec![id.to_string().as_arg(), idx.as_arg()],
                type_args: None,
            });

            let span = member.span();
            if span != DUMMY_SP {
                docblock.insert(member.span(), self.get_element_docblock(member.span()));
            }

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
                _ => {
                    // Do nothing
                }
            }
        }

        register_class(
            &id,
            ReflectionData::new(n, name, self.filename, self.namespace, docblock),
        );
    }
}

impl<C: Comments> VisitMut for ClassReflectionDecorators<'_, C> {
    noop_visit_mut_type!();

    fn visit_mut_class_decl(&mut self, n: &mut ClassDecl) {
        self.process_class(&mut n.class, n.ident.clone());
        n.visit_mut_children_with(self);
    }

    fn visit_mut_class_expr(&mut self, n: &mut ClassExpr) {
        let Some(ident) = n.ident.clone() else { panic!("anonymous_expr transformer must be called before class_reflection_decorator"); };
        self.process_class(&mut n.class, ident);
        n.visit_mut_children_with(self);
    }
}
