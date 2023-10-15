use std::path::Path;
use swc_common::{DUMMY_SP, Mark};
use swc_ecma_ast::*;
use swc_ecma_utils::{ExprFactory, prepend_stmts, quote_ident};
use swc_ecma_visit::{as_folder, Fold, noop_visit_mut_type, VisitMut, VisitMutWith};

pub fn inject_helpers(global_mark: Mark) -> impl Fold + VisitMut {
    as_folder(InjectHelpers {
        global_mark,
    })
}

struct InjectHelpers {
    global_mark: Mark,
}

impl InjectHelpers {
    fn build_helper_path(name: &str) -> String {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("lib")
            .join(&format!("_{}.js", name))
            .to_string_lossy()
            .to_string()
    }

    fn build_import(&self, name: &str, mark: Mark) -> ModuleItem {
        let s = ImportSpecifier::Named(ImportNamedSpecifier {
            span: DUMMY_SP,
            local: Ident::new(
                format!("_{}", name).into(),
                DUMMY_SP.apply_mark(mark),
            ),
            imported: Some(quote_ident!("_").into()),
            is_type_only: false,
        });

        let src: Str = Self::build_helper_path(name).into();

        ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
            span: DUMMY_SP,
            specifiers: vec![s],
            src: Box::new(src),
            with: Default::default(),
            type_only: Default::default(),
        }))
    }

    fn build_require(&self, name: &str, mark: Mark) -> Stmt {
        let c = CallExpr {
            span: DUMMY_SP,
            callee: Expr::Ident(Ident {
                span: DUMMY_SP.apply_mark(self.global_mark),
                sym: "require".into(),
                optional: false,
            })
                .as_callee(),
            args: vec![Str {
                span: DUMMY_SP,
                value: Self::build_helper_path(name).into(),
                raw: None,
            }
                .as_arg()],
            type_args: None,
        };
        let decl = Decl::Var(
            VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Var,
                declare: false,
                decls: vec![VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(
                        Ident::new(format!("_{}", name).into(), DUMMY_SP.apply_mark(mark)).into(),
                    ),
                    init: Some(c.into()),
                    definite: false,
                }],
            }
                .into(),
        );
        Stmt::Decl(decl)
    }
}

impl VisitMut for InjectHelpers {
    noop_visit_mut_type!();

    fn visit_mut_script(&mut self, script: &mut Script) {
        let helpers = vec![self.build_require("apply_decs_2203_r", self.global_mark)];
        let helpers_is_empty = helpers.is_empty();

        prepend_stmts(&mut script.body, helpers.into_iter());

        if !helpers_is_empty {
            script.visit_mut_children_with(self);
        }
    }

    fn visit_mut_module(&mut self, n: &mut Module) {
        let helpers = vec![self.build_import("apply_decs_2203_r", self.global_mark)];
        let helpers_is_empty = helpers.is_empty();

        prepend_stmts(&mut n.body, helpers.into_iter());

        if !helpers_is_empty {
            n.visit_mut_children_with(self);
        }
    }
}
