use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;

pub(crate) fn ident(word: &str) -> Ident {
    Ident::new(word.into(), DUMMY_SP, SyntaxContext::empty())
}

pub(crate) fn ident_name(word: &str) -> IdentName {
    IdentName::new(word.into(), DUMMY_SP)
}
