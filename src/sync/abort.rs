use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type AbortSignal;

    #[wasm_bindgen(structural, method, getter)]
    pub fn aborted (this: &AbortSignal) -> bool;
    #[wasm_bindgen(structural, method, getter)]
    pub fn reason (this: &AbortSignal) -> JsValue;
    #[wasm_bindgen(structural, method)]
    pub fn abort (this: &AbortSignal) -> AbortSignal;
    #[wasm_bindgen(structural, method, getter)]
    pub fn timeout (this: &AbortSignal, millis: f64) -> AbortSignal;
}