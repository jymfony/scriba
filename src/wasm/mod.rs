mod compile;
mod reflection;
mod stack_trace;

extern crate alloc;

use crate::wasm::stack_trace::{prepare_stack_trace, CallSite, Error};
use js_sys::{Array, Function};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[cfg(debug_assertions)]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::wasm::log(&format_args!($($t)*).to_string()))
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => {};
}

#[wasm_bindgen(start)]
pub fn start() {
    let previous: Option<Function> = Error::prepare_stack_trace();

    let a = Closure::<dyn Fn(Error, Box<[CallSite]>) -> String>::new(
        move |error: Error, stack: Box<[CallSite]>| -> String {
            let prev = if let Some(func) = &previous {
                let this = &JsValue::null();
                let e = JsValue::from(&error);
                let s = Array::from_iter(stack.iter());

                if let Ok(val) = func.call2(this, &e, &s) {
                    val.as_string()
                } else {
                    None
                }
            } else {
                None
            };

            prepare_stack_trace(error, stack, prev)
        },
    );

    Error::set_stack_trace_limit(0);
    Error::set_prepare_stack_trace(a.as_ref().unchecked_ref());
    a.forget();
}
