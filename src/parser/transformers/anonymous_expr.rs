use rand::Rng;
use swc_ecma_ast::*;
use swc_ecma_utils::private_ident;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

pub fn anonymous_expr() -> impl VisitMut + Fold {
    as_folder(AnonymousExpr::default())
}

#[derive(Default)]
struct AnonymousExpr {}

fn gen_anonymous_ident() -> Ident {
    let rnd = rand::thread_rng().gen_range(0..1000000);
    let ident = format!("_anonymous_xΞ{:X}", rnd);

    private_ident!(ident)
}

impl VisitMut for AnonymousExpr {
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, e: &mut Expr) {
        match e {
            Expr::Class(c) => {
                if c.ident.is_none() {
                    c.ident = Some(gen_anonymous_ident());
                }
            }
            Expr::Fn(f) => {
                if f.ident.is_none() {
                    f.ident = Some(gen_anonymous_ident());
                }
            }
            _ => {}
        }
    }
}
