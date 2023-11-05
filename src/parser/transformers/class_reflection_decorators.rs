use crate::generate_uuid;
use crate::parser::util::ident;
use crate::reflection::{register_class, ReflectionData};
use rustc_hash::FxHashMap;
use std::rc::Rc;
use swc_common::comments::{CommentKind, Comments};
use swc_common::{Span, Spanned, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_utils::{undefined, ExprFactory};
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
                if cmt.kind == CommentKind::Block && cmt.text.starts_with('*') {
                    Some(format!("/*{}*/", cmt.text))
                } else {
                    None
                }
            })
    }

    fn process_class(&self, n: &mut Class, name: Ident, outer_docblock: Option<String>) {
        let id = generate_uuid();
        let mut docblock = FxHashMap::default();
        if let Some(outer_db) = outer_docblock {
            docblock.insert(n.span, Some(outer_db));
        }

        if n.span != DUMMY_SP {
            if let Some(db) = self.get_element_docblock(n.span) {
                docblock.insert(n.span, Some(db));
            }
        }

        n.decorators.push(Decorator {
            span: DUMMY_SP,
            expr: Box::new(Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: ident("__jymfony_reflect").as_callee(),
                args: vec![
                    id.to_string().as_arg(),
                    n.body
                        .iter()
                        .enumerate()
                        .find(|(_, m)| matches!(m, ClassMember::Constructor(_)))
                        .map(|(idx, _)| idx.as_arg())
                        .unwrap_or_else(|| undefined(DUMMY_SP).as_arg()),
                ],
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

    fn visit_mut_module_item(&mut self, n: &mut ModuleItem) {
        match n {
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(ExportDefaultDecl {
                decl: DefaultDecl::Class(expr),
                span,
            })) => {
                let Some(ident) = expr.ident.clone() else {
                    panic!("anonymous_expr transformer must be called before class_reflection_decorator");
                };

                self.process_class(&mut expr.class, ident, self.get_element_docblock(*span));
                expr.visit_mut_children_with(self);
            }
            ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                decl: Decl::Class(decl),
                span,
            })) => {
                self.process_class(
                    &mut decl.class,
                    decl.ident.clone(),
                    self.get_element_docblock(*span),
                );
                decl.visit_mut_children_with(self);
            }
            _ => {
                n.visit_mut_children_with(self);
            }
        }
    }

    fn visit_mut_class_decl(&mut self, n: &mut ClassDecl) {
        self.process_class(&mut n.class, n.ident.clone(), None);
        n.visit_mut_children_with(self);
    }

    fn visit_mut_class_expr(&mut self, n: &mut ClassExpr) {
        let Some(ident) = n.ident.clone() else {
            panic!("anonymous_expr transformer must be called before class_reflection_decorator");
        };

        self.process_class(&mut n.class, ident, None);
        n.visit_mut_children_with(self);
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::transformers::class_reflection_decorators;
    use crate::testing::compile_tr;
    use swc_common::{chain, Mark};
    use swc_ecma_transforms_base::resolver;
    use swc_ecma_transforms_testing::Tester;
    use swc_ecma_visit::Fold;

    fn create_pass(tester: &mut Tester) -> Box<dyn Fold> {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        Box::new(chain!(
            resolver(unresolved_mark, top_level_mark, false),
            class_reflection_decorators(None, None, tester.comments.clone()),
        ))
    }

    #[test]
    pub fn should_compile_as_function_correctly() {
        let code = r#"
export default class TestClass {
    publicMethod(a, b = 12, c = {}) {
        console.log('test');
    }
}
"#;

        let compiled = compile_tr(|tester| create_pass(tester), code);
        assert_eq!(
            compiled,
            r#"export default @__jymfony_reflect("00000000-0000-0000-0000-000000000000", void 0)
class TestClass {
    @__jymfony_reflect("00000000-0000-0000-0000-000000000000", 0)
    publicMethod(a, b = 12, c = {}) {
        console.log('test');
    }
}
"#
        );
    }
}
