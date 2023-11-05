use crate::parser::util::*;
use lazy_static::lazy_static;
use swc_common::util::take::Take;
use swc_common::{Mark, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

lazy_static! {}

pub fn wrap_in_function(top_level_mark: Mark) -> impl VisitMut + Fold {
    as_folder(WrapInFunction {
        unresolved_mark: top_level_mark,
    })
}

#[derive(Default)]
struct WrapInFunction {
    unresolved_mark: Mark,
}

impl VisitMut for WrapInFunction {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, n: &mut Module) {
        n.visit_mut_children_with(self);

        let stmts = n.body.take();
        debug_assert!(
            stmts.iter().all(|m| m.is_stmt()),
            "must be called after commonjs transformation"
        );

        let span = DUMMY_SP.apply_mark(self.unresolved_mark);

        let wrapped = Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(Expr::Paren(ParenExpr {
                span: DUMMY_SP,
                expr: Box::new(Expr::Fn(FnExpr {
                    ident: None,
                    function: Box::new(Function {
                        params: vec![
                            Param {
                                span,
                                decorators: vec![],
                                pat: Pat::Ident(ident("exports").into()),
                            },
                            Param {
                                span,
                                decorators: vec![],
                                pat: Pat::Ident(ident("require").into()),
                            },
                            Param {
                                span,
                                decorators: vec![],
                                pat: Pat::Ident(ident("module").into()),
                            },
                            Param {
                                span,
                                decorators: vec![],
                                pat: Pat::Ident(ident("__filename").into()),
                            },
                            Param {
                                span,
                                decorators: vec![],
                                pat: Pat::Ident(ident("__dirname").into()),
                            },
                        ],
                        decorators: vec![],
                        span: DUMMY_SP,
                        body: Some(BlockStmt {
                            span: DUMMY_SP,
                            stmts: stmts.into_iter().map(|m| m.expect_stmt()).collect(),
                        }),
                        is_generator: false,
                        is_async: false,
                        type_params: None,
                        return_type: None,
                    }),
                })),
            })),
        });

        n.body = vec![ModuleItem::Stmt(wrapped)];
    }

    fn visit_mut_script(&mut self, n: &mut Script) {
        n.visit_mut_children_with(self);

        let body = n.body.take();
        let wrapped = Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(Expr::Paren(ParenExpr {
                span: DUMMY_SP,
                expr: Box::new(Expr::Fn(FnExpr {
                    ident: None,
                    function: Box::new(Function {
                        params: vec![
                            Param {
                                span: DUMMY_SP,
                                decorators: vec![],
                                pat: Pat::Ident(ident("exports").into()),
                            },
                            Param {
                                span: DUMMY_SP,
                                decorators: vec![],
                                pat: Pat::Ident(ident("require").into()),
                            },
                            Param {
                                span: DUMMY_SP,
                                decorators: vec![],
                                pat: Pat::Ident(ident("module").into()),
                            },
                            Param {
                                span: DUMMY_SP,
                                decorators: vec![],
                                pat: Pat::Ident(ident("__filename").into()),
                            },
                            Param {
                                span: DUMMY_SP,
                                decorators: vec![],
                                pat: Pat::Ident(ident("__dirname").into()),
                            },
                        ],
                        decorators: vec![],
                        span: DUMMY_SP,
                        body: Some(BlockStmt {
                            span: DUMMY_SP,
                            stmts: body,
                        }),
                        is_generator: false,
                        is_async: false,
                        type_params: None,
                        return_type: None,
                    }),
                })),
            })),
        });

        n.body = vec![wrapped];
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::transformers::wrap_in_function;
    use crate::testing::compile_tr;
    use swc_common::comments::SingleThreadedComments;
    use swc_common::{chain, Mark};
    use swc_ecma_transforms_base::resolver;
    use swc_ecma_transforms_compat::es2022::static_blocks;
    use swc_ecma_transforms_module::common_js;
    use swc_ecma_visit::Fold;

    fn create_pass() -> Box<dyn Fold> {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        let static_block_mark = Mark::new();

        Box::new(chain!(
            resolver(unresolved_mark, top_level_mark, false),
            common_js::<SingleThreadedComments>(
                unresolved_mark,
                Default::default(),
                Default::default(),
                None
            ),
            wrap_in_function(top_level_mark),
            static_blocks(static_block_mark),
        ))
    }

    #[test]
    pub fn should_compile_as_function_correctly() {
        let code = r#"
export default class TestClass {
}
"#;

        let compiled = compile_tr(|_| create_pass(), code);
        assert_eq!(
            compiled,
            r#"(function(exports, require, module, __filename, __dirname) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "default", {
        enumerable: true,
        get: function() {
            return TestClass;
        }
    });
    class TestClass {
    }
});
"#
        );
    }
}
