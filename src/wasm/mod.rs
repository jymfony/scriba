extern crate alloc;

use crate::stack::remap_stack_trace;
use crate::Frame;
use js_sys::*;
use std::iter::Iterator;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Error")]
    pub type Error;

    #[wasm_bindgen(method, getter)]
    pub fn name(this: &Error) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn message(this: &Error) -> JsValue;

    #[wasm_bindgen(method, getter)]
    pub fn stack(this: &Error) -> Option<String>;

    #[wasm_bindgen(static_method_of = Error, getter = prepareStackTrace)]
    pub fn prepare_stack_trace() -> Option<Function>;
    #[wasm_bindgen(static_method_of = Error, setter = prepareStackTrace)]
    pub fn set_prepare_stack_trace(closure: &Function);

    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Error) -> String;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "NodeJS.CallSite")]
    pub type CallSite;

    /// Value of "this"
    #[wasm_bindgen(method, js_name = getThis)]
    pub fn get_this(this: &CallSite) -> JsValue;

    /// Type of "this" as a string.
    /// This is the name of the function stored in the constructor field of
    /// "this", if available.  Otherwise the object's [[Class]] internal
    /// property.
    #[wasm_bindgen(method, js_name = getTypeName)]
    pub fn get_type_name(this: &CallSite) -> Option<String>;

    /// Current function
    #[wasm_bindgen(method, js_name = getFunction)]
    pub fn get_function(this: &CallSite) -> Option<Function>;

    /// Name of the current function, typically its name property.
    /// If a name property is not available an attempt will be made to try
    /// to infer a name from the function's context.
    #[wasm_bindgen(method, js_name = getFunctionName)]
    pub fn get_function_name(this: &CallSite) -> Option<String>;

    /// Name of the property [of "this" or one of its prototypes] that holds
    /// the current function
    #[wasm_bindgen(method, js_name = getMethodName)]
    pub fn get_method_name(this: &CallSite) -> Option<String>;

    /// Name of the script [if this function was defined in a script]
    #[wasm_bindgen(method, js_name = getFileName)]
    pub fn get_file_name(this: &CallSite) -> Option<String>;

    /// Current line number [if this function was defined in a script]
    #[wasm_bindgen(method, js_name = getLineNumber)]
    pub fn get_line_number(this: &CallSite) -> Option<i32>;

    /// Current column number [if this function was defined in a script]
    #[wasm_bindgen(method, js_name = getColumnNumber)]
    pub fn get_column_number(this: &CallSite) -> Option<i32>;

    /// A call site object representing the location where eval was called
    /// [if this function was created using a call to eval]
    #[wasm_bindgen(method, js_name = getEvalOrigin)]
    pub fn get_eval_origin(this: &CallSite) -> Option<String>;

    /// Is this a toplevel invocation, that is, is "this" the global object?
    #[wasm_bindgen(method, js_name = isToplevel)]
    pub fn is_top_level(this: &CallSite) -> bool;

    /// Does this call take place in code defined by a call to eval?
    #[wasm_bindgen(method, js_name = isEval)]
    pub fn is_eval(this: &CallSite) -> bool;

    /// Is this call in native V8 code?
    #[wasm_bindgen(method, js_name = isNative)]
    pub fn is_native(this: &CallSite) -> bool;

    /// Is this a constructor call?
    #[wasm_bindgen(method, js_name = isConstructor)]
    pub fn is_constructor(this: &CallSite) -> bool;

    /// Is this an async call (i.e. await, Promise.all(), or Promise.any())?
    #[wasm_bindgen(method, js_name = isAsync)]
    pub fn is_async(this: &CallSite) -> bool;

    /// Is this an async call to Promise.all()?
    #[wasm_bindgen(method, js_name = isPromiseAll)]
    pub fn is_promise_all(this: &CallSite) -> bool;

    /// Is this an async call to Promise.any()?
    #[wasm_bindgen(method, js_name = isPromiseAny)]
    pub fn is_promise_any(this: &CallSite) -> bool;

    /// Returns the index of the promise element that was followed in
    /// Promise.all() or Promise.any() for async stack traces, or null if the
    /// CallSite is not an async Promise.all() or Promise.any() call.
    #[wasm_bindgen(method, js_name = getPromiseIndex)]
    pub fn get_promise_index(this: &CallSite) -> Option<usize>;

    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &CallSite) -> String;
}

impl From<&CallSite> for Frame {
    fn from(value: &CallSite) -> Self {
        Self {
            filename: value.get_file_name(),
            line_no: value.get_line_number().unwrap_or_default() as u32,
            col_no: value.get_column_number().unwrap_or_default() as u32,
            function_name: value.get_function_name(),
            method_name: value.get_method_name(),
            type_name: value.get_type_name(),
            is_native: value.is_native(),
            is_top_level: value.is_top_level(),
            is_constructor: value.is_constructor(),
            is_async: value.is_async(),
            is_promise_all: value.is_promise_all(),
            promise_index: value.get_promise_index().unwrap_or_default(),
            string_repr: value.to_string(),
        }
    }
}

#[wasm_bindgen(js_name = prepareStackTrace)]
pub fn prepare_stack_trace(
    error: Error,
    stack: Box<[CallSite]>,
    previous: Option<String>,
) -> String {
    let message: JsValue = error.message();
    let message: String = message
        .dyn_ref::<JsString>()
        .cloned()
        .unwrap_or_else(|| JsString::from(""))
        .into();

    let stack = stack
        .into_iter()
        .map(|cs| Frame::from(cs))
        .collect::<Vec<_>>();
    remap_stack_trace(&message, stack.into_boxed_slice(), previous)
}
