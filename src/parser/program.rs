use crate::parser::transformers::{
    anonymous_expr, class_jobject, class_reflection_decorators, decorator_2022_03,
};
use crate::stack::register_source_map;
use sourcemap::SourceMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use swc_common::comments::SingleThreadedComments;
use swc_common::{chain, BytePos, LineCol, Mark, GLOBALS};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::Emitter;
use swc_ecma_transforms_base::feature::FeatureFlag;
use swc_ecma_transforms_base::hygiene::hygiene;
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_compat::es2022::static_blocks;
use swc_ecma_transforms_module::common_js;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

#[derive(Debug)]
pub struct CompileOptions {
    debug: bool,
}

pub struct Program {
    pub(crate) source_map: Arc<swc_common::SourceMap>,
    pub(crate) orig_srcmap: Option<SourceMap>,
    pub(crate) filename: Option<String>,
    pub(crate) program: swc_ecma_ast::Program,
    pub(crate) comments: SingleThreadedComments,
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

            let mut transformers = chain!(
                resolver(unresolved_mark, top_level_mark, self.is_typescript),
                strip(top_level_mark),
                anonymous_expr(),
                class_reflection_decorators(),
                class_jobject(),
                decorator_2022_03(),
                static_blocks(static_block_mark),
                hygiene(),
                common_js(
                    unresolved_mark,
                    Default::default(),
                    available_set,
                    Some(&self.comments)
                ),
            );

            if !opts.debug {
                // TODO
                // transformers = chain!(transformers, remove_assert_calls());
            }

            let program = self.program.fold_with(&mut transformers);
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

            if let Some(f) = self.filename {
                let srcmap = self
                    .source_map
                    .build_source_map_from(&sm, self.orig_srcmap.as_ref());
                register_source_map(f, srcmap);
            }

            Ok(String::from_utf8(buf).expect("non-utf8?"))
        })
    }
}
