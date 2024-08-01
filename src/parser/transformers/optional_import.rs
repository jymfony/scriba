use crate::parser::util::ident_name;
use swc_atoms::JsWord;
use swc_common::util::take::Take;
use swc_common::{Mark, Span, SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_transforms_base::enable_helper;
use swc_ecma_utils::{private_ident, quote_ident, ExprFactory};
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
            callee: quote_ident!(
                SyntaxContext::empty().apply_mark(unresolved_mark),
                "require"
            )
            .as_callee(),
            args: vec![Lit::Str(Str {
                span: src_span,
                raw: None,
                value: src,
            })
            .as_arg()],

            ..Default::default()
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
                ..Default::default()
            },
            handler: Some(CatchClause {
                span: DUMMY_SP,
                param: None,
                body: BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![Stmt::Return(ReturnStmt {
                        span: DUMMY_SP,
                        arg: Some(Expr::undefined(DUMMY_SP)),
                    })],
                    ..Default::default()
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
                        && kv.value.as_lit().is_some_and(|l| match l {
                            Lit::Bool(b) => b.value,
                            Lit::Str(s) => s.value == "true",
                            _ => false,
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
                                                ..Default::default()
                                            }),
                                            is_generator: false,
                                            is_async: false,
                                            ..Default::default()
                                        }),
                                    })
                                    .as_iife(),
                                ))),
                                definite: false,
                            }],
                            ..Default::default()
                        }))));

                        for spec in import.specifiers.into_iter() {
                            match spec {
                                ImportSpecifier::Namespace(ImportStarAsSpecifier {
                                    local,
                                    span,
                                }) => {
                                    let mark = enable_helper!(interop_require_wildcard);
                                    let ctxt = SyntaxContext::empty().apply_mark(mark);

                                    let call_expr = Expr::from(quote_ident!(
                                        ctxt,
                                        span,
                                        "_interop_require_wildcard"
                                    ))
                                    .as_call(span, vec![req.clone().as_arg(), true.as_arg()]);

                                    let ternary = Expr::Cond(CondExpr {
                                        span,
                                        test: BinExpr {
                                            span,
                                            op: BinaryOp::EqEqEq,
                                            left: Expr::undefined(DUMMY_SP),
                                            right: Box::new(Expr::Ident(req.clone())),
                                        }
                                        .into(),
                                        cons: call_expr.into(),
                                        alt: Expr::undefined(DUMMY_SP),
                                    });

                                    stmts.push(
                                        VarDecl {
                                            span: DUMMY_SP,
                                            kind: VarDeclKind::Const,
                                            declare: false,
                                            decls: vec![VarDeclarator {
                                                span: DUMMY_SP,
                                                name: Pat::Ident(local.into()),
                                                init: Some(Box::new(ternary)),
                                                definite: false,
                                            }],
                                            ..Default::default()
                                        }
                                        .into(),
                                    );
                                }
                                ImportSpecifier::Default(ImportDefaultSpecifier {
                                    local,
                                    span,
                                }) => {
                                    let mark = enable_helper!(interop_require_default);
                                    let ctxt = SyntaxContext::empty().apply_mark(mark);

                                    let call_expr = Expr::from(quote_ident!(
                                        ctxt,
                                        span,
                                        "_interop_require_default"
                                    ))
                                    .as_call(span, vec![req.clone().as_arg(), true.as_arg()]);

                                    let ternary = Expr::Cond(CondExpr {
                                        span,
                                        test: BinExpr {
                                            span,
                                            op: BinaryOp::NotEqEq,
                                            left: Expr::undefined(DUMMY_SP),
                                            right: Box::new(Expr::Ident(req.clone())),
                                        }
                                        .into(),
                                        cons: MemberExpr {
                                            span,
                                            obj: Box::new(call_expr),
                                            prop: MemberProp::Ident(ident_name("default")),
                                        }
                                        .into(),
                                        alt: Expr::undefined(DUMMY_SP),
                                    });

                                    stmts.push(
                                        VarDecl {
                                            span: DUMMY_SP,
                                            kind: VarDeclKind::Const,
                                            declare: false,
                                            decls: vec![VarDeclarator {
                                                span: DUMMY_SP,
                                                name: Pat::Ident(local.into()),
                                                init: Some(ternary.into()),
                                                definite: false,
                                            }],
                                            ..Default::default()
                                        }
                                        .into(),
                                    );
                                }
                                ImportSpecifier::Named(ImportNamedSpecifier {
                                    local,
                                    imported,
                                    span,
                                    ..
                                }) => {
                                    let prop = match imported {
                                        None => MemberProp::Ident(local.clone().into()),
                                        Some(ModuleExportName::Ident(i)) => {
                                            MemberProp::Ident(i.into())
                                        }
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

                                    stmts.push(
                                        VarDecl {
                                            span: DUMMY_SP,
                                            kind: VarDeclKind::Const,
                                            declare: false,
                                            decls: vec![VarDeclarator {
                                                span: DUMMY_SP,
                                                name: Pat::Ident(local.into()),
                                                init: Some(Box::new(access)),
                                                definite: false,
                                            }],
                                            ..Default::default()
                                        }
                                        .into(),
                                    );
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

#[cfg(test)]
mod tests {
    use crate::parser::transformers::optional_import;
    use crate::testing::compile_tr;
    use swc_common::{chain, Mark};
    use swc_ecma_transforms_base::resolver;
    use swc_ecma_visit::Fold;

    fn create_pass() -> Box<dyn Fold> {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        Box::new(chain!(
            resolver(unresolved_mark, top_level_mark, false),
            optional_import(unresolved_mark),
        ))
    }

    #[test]
    pub fn should_compile_optional_imports_correctly() {
        let code = r#"
import Redis, { Cluster as RedisCluster } from 'ioredis' with { optional: 'true' };
class RedisAdapter {
}
"#;

        let compiled = compile_tr(|_| create_pass(), code);
        assert_eq!(
            compiled,
            r#"const _r = function() {
    try {
        return require("ioredis");
    } catch  {
        return void 0;
    }
}();
const Redis = void 0 !== _r ? _interop_require_default(_r, true).default : void 0;
const RedisCluster = _r?.Cluster;
;
class RedisAdapter {
}
"#
        );
    }
}
