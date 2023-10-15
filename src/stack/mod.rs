mod trace;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;

pub(crate) use trace::remap_stack_trace;

struct InternalSourceMap(sourcemap::SourceMap);
unsafe impl Send for InternalSourceMap {}
unsafe impl Sync for InternalSourceMap {}

lazy_static! {
    static ref FILE_MAPPINGS: RwLock<HashMap<String, InternalSourceMap>> =
        RwLock::new(HashMap::new());
}

pub(crate) struct Frame {
    pub filename: Option<String>,
    pub line_no: u32,
    pub col_no: u32,
    pub function_name: Option<String>,
    pub method_name: Option<String>,
    pub type_name: Option<String>,
    pub is_native: bool,
    pub is_top_level: bool,
    pub is_constructor: bool,
    pub is_async: bool,
    pub is_promise_all: bool,
    pub promise_index: usize,
    pub string_repr: String,
}

pub fn register_source_map(filename: String, srcmap: sourcemap::SourceMap) {
    let mut mappings = FILE_MAPPINGS.write().unwrap();
    mappings.insert(filename, InternalSourceMap(srcmap));
}
