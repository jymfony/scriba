use crate::parser::{parse, CompileOptions};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const ITEXT_STYLE: &'static str = r#"
interface CompileOptions {
    debug?: boolean;
    namespace?: string;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "CompileOptions")]
    pub type WasmCompileOptions;

    #[wasm_bindgen(structural, method, getter)]
    fn debug(this: &WasmCompileOptions) -> Option<bool>;

    #[wasm_bindgen(structural, method, getter)]
    fn namespace(this: &WasmCompileOptions) -> Option<String>;
}

#[wasm_bindgen(js_name = compile)]
pub fn compile(
    source: String,
    filename: Option<String>,
    opts: Option<WasmCompileOptions>,
) -> Result<String, JsError> {
    let debug = opts.as_ref().and_then(|c| c.debug()).unwrap_or_default();
    let namespace = opts.as_ref().and_then(|c| c.namespace());

    let program = match parse(source, filename.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            return Err(JsError::new(&e.to_string()));
        }
    };

    Ok(program.compile(CompileOptions { debug, namespace })?)
}
