use crate::parser::util::*;
use lazy_static::lazy_static;
use swc_common::util::take::Take;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

lazy_static! {}

pub fn wrap_in_function() -> impl VisitMut + Fold {
    as_folder(WrapInFunction::default())
}

#[derive(Default)]
struct WrapInFunction {}

impl VisitMut for WrapInFunction {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, n: &mut Module) {
        n.visit_mut_children_with(self);

        let stmts = n.body.take();
        debug_assert!(
            stmts.iter().all(|m| m.is_stmt()),
            "must be called after commonjs transformation"
        );

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
