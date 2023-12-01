#[macro_use]
extern crate lazy_static;

mod viewmodel;

use std::f64;
use wasm_bindgen::prelude::*;
use wasm_bindgen::__rt::std::sync::{Arc, RwLock};
use crate::viewmodel::{InterpreterViewModel, ViewModel};
use wasm_bindgen::JsCast;
use web_sys;
use js_sys;
use wasm_bindgen_futures::JsFuture;
use interpreter::Interpreter;

#[wasm_bindgen(inline_js = "module.exports.sleep = function sleep(ms) { return new Promise((resolve)=> setTimeout(resolve, ms)); }")]
extern "C"  {
    fn sleep(ms: f64) -> js_sys::Promise;
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

lazy_static! {
    pub static ref interp: Arc<RwLock<Interpreter>> = Arc::new(RwLock::new(Interpreter::new()));
}

#[wasm_bindgen]
pub async fn run(f: JsValue) {
    let view_model = InterpreterViewModel::new(interp.clone());

    let f = js_sys::Function::from(f);

    loop {
        let visual_objects =
            JsValue::from_serde(&view_model.visual_objects()).unwrap();
        f.call1(&JsValue::NULL, &visual_objects).unwrap();
        JsFuture::from(sleep(1000.0)).await.unwrap();
    }
}


#[wasm_bindgen]
pub fn exec(code: &str) {
    interp.write().unwrap().exec(code);
}
