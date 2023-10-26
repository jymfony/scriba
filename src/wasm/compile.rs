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

#[cfg(test)]
mod tests {
    use crate::parse_uuid;
    use crate::parser::{parse, CompileOptions};
    use crate::reflection::get_reflection_data;
    use crate::testing::uuid::reset_test_uuid;
    use crate::wasm::reflection::get_js_reflection_data;

    #[test]
    pub fn should_compile_program_correctly() -> Result<(), Box<dyn std::error::Error>> {
        reset_test_uuid();

        let code = r#"
const a = () => Symbol('test');
const type = t => {
    return function (value, context) {
        console.log(value, context);
    };
};

/** class docblock */
export default class x {
    static #staticPrivateField;
    #privateField;
    accessor #privateAccessor;
    static staticPublicField;
    publicField;
    accessor publicAccessor;

    /**
     * computed method docblock
     */
    [a()]() {}
    #privateMethod(a, b = 1, [c, d], {f, g}) {}
    publicMethod({a, b} = {}, c = new Object(), ...x) {}
    static #staticPrivateMethod() {}
    static staticPublicMethod() {}

    get [a()]() {}
    set b(v) {}

    get #ap() {}
    set #bp(v) {}

    act(@type(String) param1) {}
    [a()](@type(String) param1) {}
    [Symbol.for('xtest')](@type(String) param1) {}
}
"#;

        let program = parse(code.to_string(), Some("x.js"))?;
        let compiled = program.compile(CompileOptions {
            debug: true,
            namespace: None,
        })?;

        let data = get_js_reflection_data("00000000-0000-0000-0000-000000000000");

        Ok(())
    }
}
