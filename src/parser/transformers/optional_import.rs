use crate::parser::util::ident;
use swc_atoms::JsWord;
use swc_common::util::take::Take;
use swc_common::{Mark, Span, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_transforms_base::enable_helper;
use swc_ecma_utils::{private_ident, quote_ident, undefined, ExprFactory};
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

pub fn optional_import(unresolved_mark: Mark) -> impl VisitMut + Fold {
    as_folder(OptionalImport {
        unresolved_mark,
        ..Default::default()
    })
}

#[derive(Default)]
struct OptionalImport {
    unresolved_mark: Mark,
    optional_imports: Vec<ModuleItem>,
}

impl OptionalImport {
    pub(crate) fn make_require_call(
        &self,
        unresolved_mark: Mark,
        src: JsWord,
        src_span: Span,
    ) -> Expr {
        Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: quote_ident!(DUMMY_SP.apply_mark(unresolved_mark), "require").as_callee(),
            args: vec![Lit::Str(Str {
                span: src_span,
                raw: None,
                value: src,
            })
            .as_arg()],

            type_args: Default::default(),
        })
    }
}

impl OptionalImport {
    fn wrap_in_try(expr: Expr) -> Stmt {
        Stmt::Try(Box::new(TryStmt {
            span: DUMMY_SP,
            block: BlockStmt {
                span: DUMMY_SP,
                stmts: vec![Stmt::Return(ReturnStmt {
                    span: DUMMY_SP,
                    arg: Some(Box::new(expr)),
                })],
            },
            handler: Some(CatchClause {
                span: DUMMY_SP,
                param: None,
                body: BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![Stmt::Return(ReturnStmt {
                        span: DUMMY_SP,
                        arg: Some(undefined(DUMMY_SP)),
                    })],
                },
            }),
            finalizer: None,
        }))
    }
}

impl VisitMut for OptionalImport {
    noop_visit_mut_type!();

    fn visit_mut_module_item(&mut self, n: &mut ModuleItem) {
        n.visit_mut_children_with(self);

        let ModuleItem::ModuleDecl(ModuleDecl::Import(i)) = n else {
            return;
        };
        let Some(w) = &i.with else {
            return;
        };

        let optional = w.props.iter().find(|p| {
            p.as_prop()
                .and_then(|p| p.as_key_value())
                .map(|kv| {
                    kv.key.as_ident().is_some_and(|i| i.sym == "optional")
                        && kv.value.as_lit().is_some_and(|l| {
                            let Lit::Bool(b) = l else {
                                return false;
                            };
                            b.value
                        })
                })
                .unwrap_or(false)
        });

        if optional.is_some() {
            self.optional_imports.push(n.take());
        }
    }

    fn visit_mut_module_items(&mut self, n: &mut Vec<ModuleItem>) {
        n.visit_mut_children_with(self);

        let el = n
            .iter()
            .enumerate()
            .find(|(_, item)| matches!(item, ModuleItem::Stmt(_)));

        let index = if let Some((idx, _)) = el {
            idx
        } else {
            n.len()
        };

        n.splice(
            index..index,
            self.optional_imports
                .take()
                .into_iter()
                .flat_map(|item| {
                    let decl = item.expect_module_decl();
                    let import = decl.expect_import();

                    let mut stmts = vec![];

                    let require_call = self.make_require_call(
                        self.unresolved_mark,
                        import.src.value,
                        import.src.span,
                    );
                    if import.specifiers.is_empty() {
                        stmts.push(Self::wrap_in_try(require_call));
                    } else {
                        let req = private_ident!("_r");
                        let call_req = Self::wrap_in_try(require_call);

                        stmts.push(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                            span: DUMMY_SP,
                            kind: VarDeclKind::Const,
                            declare: false,
                            decls: vec![VarDeclarator {
                                span: DUMMY_SP,
                                name: Pat::Ident(req.clone().into()),
                                init: Some(Box::new(Expr::Call(
                                    Expr::Fn(FnExpr {
                                        ident: None,
                                        function: Box::new(Function {
                                            params: vec![],
                                            decorators: vec![],
                                            span: Default::default(),
                                            body: Some(BlockStmt {
                                                span: DUMMY_SP,
                                                stmts: vec![call_req],
                                            }),
                                            is_generator: false,
                                            is_async: false,
                                            type_params: None,
                                            return_type: None,
                                        }),
                                    })
                                    .as_iife(),
                                ))),
                                definite: false,
                            }],
                        }))));

                        for spec in import.specifiers.into_iter() {
                            match spec {
                                ImportSpecifier::Namespace(ImportStarAsSpecifier {
                                    local,
                                    span,
                                }) => {
                                    let mark = enable_helper!(interop_require_wildcard);
                                    let span = span.apply_mark(mark);

                                    let call_expr =
                                        Expr::from(quote_ident!(span, "_interop_require_wildcard"))
                                            .as_call(
                                                span,
                                                vec![req.clone().as_arg(), true.as_arg()],
                                            );

                                    let ternary = Expr::Cond(CondExpr {
                                        span,
                                        test: Box::new(Expr::Bin(BinExpr {
                                            span,
                                            op: BinaryOp::EqEqEq,
                                            left: undefined(DUMMY_SP),
                                            right: Box::new(Expr::Ident(req.clone())),
                                        })),
                                        cons: Box::new(call_expr),
                                        alt: undefined(DUMMY_SP),
                                    });

                                    stmts.push(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                        span: DUMMY_SP,
                                        kind: VarDeclKind::Const,
                                        declare: false,
                                        decls: vec![VarDeclarator {
                                            span: DUMMY_SP,
                                            name: Pat::Ident(local.into()),
                                            init: Some(Box::new(ternary)),
                                            definite: false,
                                        }],
                                    }))));
                                }
                                ImportSpecifier::Default(ImportDefaultSpecifier {
                                    local,
                                    span,
                                }) => {
                                    let mark = enable_helper!(interop_require_default);
                                    let span = span.apply_mark(mark);

                                    let call_expr =
                                        Expr::from(quote_ident!(span, "_interop_require_default"))
                                            .as_call(
                                                span,
                                                vec![req.clone().as_arg(), true.as_arg()],
                                            );

                                    let ternary = Expr::Cond(CondExpr {
                                        span,
                                        test: Box::new(Expr::Bin(BinExpr {
                                            span,
                                            op: BinaryOp::NotEqEq,
                                            left: undefined(DUMMY_SP),
                                            right: Box::new(Expr::Ident(req.clone())),
                                        })),
                                        cons: Box::new(Expr::Member(MemberExpr {
                                            span,
                                            obj: Box::new(call_expr),
                                            prop: MemberProp::Ident(ident("default")),
                                        })),
                                        alt: undefined(DUMMY_SP),
                                    });

                                    stmts.push(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                        span: DUMMY_SP,
                                        kind: VarDeclKind::Const,
                                        declare: false,
                                        decls: vec![VarDeclarator {
                                            span: DUMMY_SP,
                                            name: Pat::Ident(local.into()),
                                            init: Some(Box::new(ternary)),
                                            definite: false,
                                        }],
                                    }))));
                                }
                                ImportSpecifier::Named(ImportNamedSpecifier {
                                    local,
                                    imported,
                                    span,
                                    ..
                                }) => {
                                    let prop = match imported {
                                        None => MemberProp::Ident(local.clone()),
                                        Some(ModuleExportName::Ident(i)) => MemberProp::Ident(i),
                                        Some(ModuleExportName::Str(s)) => {
                                            MemberProp::Computed(ComputedPropName {
                                                span,
                                                expr: Box::new(Expr::Lit(Lit::Str(s))),
                                            })
                                        }
                                    };

                                    let access = Expr::OptChain(OptChainExpr {
                                        span: DUMMY_SP,
                                        optional: true,
                                        base: Box::new(OptChainBase::Member(MemberExpr {
                                            span,
                                            obj: Box::new(Expr::Ident(req.clone())),
                                            prop,
                                        })),
                                    });

                                    stmts.push(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                        span: DUMMY_SP,
                                        kind: VarDeclKind::Const,
                                        declare: false,
                                        decls: vec![VarDeclarator {
                                            span: DUMMY_SP,
                                            name: Pat::Ident(local.into()),
                                            init: Some(Box::new(access)),
                                            definite: false,
                                        }],
                                    }))));
                                }
                            }
                        }
                    }

                    stmts
                })
                .map(ModuleItem::Stmt),
        );
    }
}
