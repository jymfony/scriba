use crate::parser::transformers::{
    anonymous_expr, class_jobject, class_reflection_decorators, decorator_2022_03,
    remove_assert_calls, resolve_self_identifiers, static_blocks,
};
use crate::stack::register_source_map;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use sourcemap::SourceMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use swc_common::comments::SingleThreadedComments;
use swc_common::sync::Lrc;
use swc_common::{chain, BytePos, LineCol, Mark, GLOBALS};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::Emitter;
use swc_ecma_transforms_base::feature::FeatureFlag;
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_transforms_base::hygiene::hygiene;
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_compat::es2020::{nullish_coalescing, optional_chaining};
use swc_ecma_transforms_module::common_js;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::{Fold, FoldWith};

#[derive(Default)]
pub struct CompileOptions {
    pub debug: bool,
    pub namespace: Option<String>,
}

pub struct Program {
    pub(crate) source_map: Lrc<swc_common::SourceMap>,
    pub(crate) orig_srcmap: Option<SourceMap>,
    pub(crate) filename: Option<String>,
    pub(crate) program: swc_ecma_ast::Program,
    pub(crate) comments: Rc<SingleThreadedComments>,
    pub(crate) is_typescript: bool,
}

impl Debug for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Program")
            .field("filename", &self.filename)
            .field("program", &self.program)
            .field("comments", &self.comments)
            .field("is_typescript", &self.is_typescript)
            .finish_non_exhaustive()
    }
}

impl Program {
    pub fn compile(self, opts: CompileOptions) -> std::io::Result<String> {
        GLOBALS.set(&Default::default(), || {
            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();
            let static_block_mark = Mark::new();
            let available_set = FeatureFlag::all();

            let mut transformers: Box<dyn Fold> = Box::new(chain!(
                resolver(unresolved_mark, top_level_mark, self.is_typescript),
                class_reflection_decorators(
                    self.filename.as_deref(),
                    opts.namespace.as_deref(),
                    self.comments.clone()
                ),
                strip(top_level_mark),
                nullish_coalescing(Default::default()),
                optional_chaining(Default::default(), unresolved_mark),
                anonymous_expr(),
                resolve_self_identifiers(unresolved_mark),
                class_jobject(),
                decorator_2022_03(),
                static_blocks(static_block_mark),
                common_js(
                    unresolved_mark,
                    Default::default(),
                    available_set,
                    Some(&self.comments)
                ),
                hygiene(),
                fixer(Some(&self.comments))
            ));

            if !opts.debug {
                transformers = Box::new(chain!(transformers, remove_assert_calls()));
            }

            let program = self.program.fold_with(transformers.as_mut());
            let mut buf = vec![];
            let mut sm: Vec<(BytePos, LineCol)> = vec![];

            {
                let mut emitter = Emitter {
                    cfg: Default::default(),
                    cm: self.source_map.clone(),
                    comments: Some(&self.comments),
                    wr: JsWriter::new(Default::default(), "\n", &mut buf, Some(&mut sm)),
                };

                emitter.emit_program(&program)?
            };

            let mut src = String::from_utf8(buf).expect("non-utf8?");
            if let Some(f) = self.filename.as_deref() {
                let srcmap = self
                    .source_map
                    .build_source_map_from(&sm, self.orig_srcmap.as_ref());

                register_source_map(f.to_string(), srcmap.clone());

                let mut buf = vec![];
                srcmap.to_writer(&mut buf).ok();

                let res = BASE64_STANDARD.encode(buf);
                src += "\n\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,";
                src += &res;
            }

            Ok(src)
        })
    }
}
