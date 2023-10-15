use crate::parser::util::*;
use lazy_static::lazy_static;
use swc_common::util::take::Take;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

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
    static ref FIELD_INITIALIZATION_SYM: MemberExpr = {
        let obj_expr = ident("Symbol");
        let prop = ident("__jymfony_field_initialization");

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
        if n.super_class.is_some() {
            return;
        }

        n.super_class = Some(Box::new(Expr::Member(JOBJECT_ACCESSOR.clone())));
        let mut stmts = vec![];
        let mut initializers = vec![];

        for mut member in n.body.drain(..) {
            match &mut member {
                ClassMember::Constructor(constructor) => {
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
                }
                ClassMember::ClassProp(prop) => {
                    if !prop.is_static {
                        initializers.push(member);
                        continue;
                    }
                }
                _ => {}
            };

            stmts.push(member);
        }

        n.body = stmts;
        if !initializers.is_empty() {
            let mut block_stmts = vec![];
            let sc_ident = ident("superCall");
            let super_call = Stmt::Decl(Decl::Var(Box::new(VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: false,
                decls: vec![VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(sc_ident.clone().into()),
                    init: Some(Box::new(Expr::SuperProp(SuperPropExpr {
                        span: DUMMY_SP,
                        obj: Super::dummy(),
                        prop: SuperProp::Computed(ComputedPropName {
                            span: Default::default(),
                            expr: Box::new(Expr::Member(FIELD_INITIALIZATION_SYM.clone())),
                        }),
                    }))),
                    definite: false,
                }],
            })));

            let if_block = Stmt::If(IfStmt {
                span: DUMMY_SP,
                test: Box::new(Expr::Bin(BinExpr {
                    span: DUMMY_SP,
                    op: BinaryOp::EqEqEq,
                    left: Box::new(undefined()),
                    right: Box::new(Expr::Ident(sc_ident.clone())),
                })),
                cons: Box::new(Stmt::Expr(ExprStmt {
                    span: DUMMY_SP,
                    expr: Box::new(Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                            span: DUMMY_SP,
                            obj: Box::new(Expr::Ident(sc_ident)),
                            prop: MemberProp::Ident(ident("apply")),
                        }))),
                        args: vec![ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::This(ThisExpr::dummy())),
                        }],
                        type_args: None,
                    })),
                })),
                alt: None,
            });

            block_stmts.push(super_call);
            block_stmts.push(if_block);

            for p in initializers.drain(..) {
                let ClassMember::ClassProp(mut p) = p else {
                    unreachable!()
                };
                assert!(!p.is_static);

                let value = p.value.take();
                let prop_name = match p.key {
                    PropName::Ident(i) => MemberProp::Ident(i),
                    PropName::Str(s) => MemberProp::Computed(ComputedPropName {
                        span: s.span,
                        expr: Box::new(Expr::Lit(Lit::Str(s))),
                    }),
                    PropName::Num(n) => MemberProp::Computed(ComputedPropName {
                        span: n.span,
                        expr: Box::new(Expr::Lit(Lit::Num(n))),
                    }),
                    PropName::Computed(c) => MemberProp::Computed(ComputedPropName {
                        span: c.span,
                        expr: c.expr,
                    }),
                    PropName::BigInt(n) => MemberProp::Computed(ComputedPropName {
                        span: n.span,
                        expr: Box::new(Expr::Lit(Lit::BigInt(n))),
                    }),
                };

                let e = Expr::Assign(AssignExpr {
                    span: DUMMY_SP,
                    op: AssignOp::Assign,
                    left: PatOrExpr::Expr(Box::new(Expr::Member(MemberExpr {
                        span: DUMMY_SP,
                        obj: Box::new(Expr::This(ThisExpr::dummy())),
                        prop: prop_name,
                    }))),
                    right: value.unwrap_or_else(|| Box::new(undefined())),
                });

                block_stmts.push(Stmt::Expr(ExprStmt {
                    span: p.span,
                    expr: Box::new(e),
                }));
            }

            n.body.push(ClassMember::Method(ClassMethod {
                span: DUMMY_SP,
                key: PropName::Computed(ComputedPropName {
                    span: DUMMY_SP,
                    expr: Box::new(Expr::Member(FIELD_INITIALIZATION_SYM.clone())),
                }),
                function: Box::new(Function {
                    params: vec![],
                    decorators: vec![],
                    span: DUMMY_SP,
                    body: Some(BlockStmt {
                        span: DUMMY_SP,
                        stmts: block_stmts,
                    }),
                    is_generator: false,
                    is_async: false,
                    type_params: None,
                    return_type: None,
                }),
                kind: MethodKind::Method,
                is_static: false,
                accessibility: None,
                is_abstract: false,
                is_optional: false,
                is_override: false,
            }));
        }
    }
}
