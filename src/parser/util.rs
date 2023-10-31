use swc_common::DUMMY_SP;
use swc_ecma_ast::*;

pub(crate) fn ident(word: &str) -> Ident {
    Ident::new(word.into(), DUMMY_SP)
}
