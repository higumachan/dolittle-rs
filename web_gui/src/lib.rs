mod viewmodel;

use std::f64;
use wasm_bindgen::prelude::*;
use interpreter::Interpreter;
use wasm_bindgen::__rt::std::sync::{Arc, RwLock};
use crate::viewmodel::InterpreterViewModel;
use wasm_bindgen::JsCast;
use web_sys;
use js_sys;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(inline_js = "module.exports.sleep = function sleep(ms) { return new Promise((resolve)=> setTimeout(resolve, ms)); }")]
extern "C"  {
    fn sleep(ms: f64) -> js_sys::Promise;
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub async fn run(f: JsValue) {
    let mut interpreter = Arc::new(RwLock::new(Interpreter::new()));
    let view_model = InterpreterViewModel::new(interpreter.clone());

    let f = js_sys::Function::from(f);

    loop {
        f.call1(&JsValue::NULL, &JsValue::from_str("test"));
        JsFuture::from(sleep(1000.0)).await;
    }
}
