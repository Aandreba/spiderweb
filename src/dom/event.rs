use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use super::DOMHighResTimeStamp;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u16)]
pub enum PhaseType {
    Capturing = 1,
    AtTarget = 2,
    Bubbling = 3,
    // todo none
}

#[wasm_bindgen]
extern "C" {
    // https://www.w3.org/TR/DOM-Level-2-Events/events.html#Events-Event
    #[derive(Debug, Clone, PartialEq)]
    pub type Event;

    #[wasm_bindgen(structural, method, getter, js_name = type)]
    pub fn ty(this: &Event) -> String;
    #[wasm_bindgen(structural, method, getter)]
    pub fn target(this: &Event) -> JsValue;
    #[wasm_bindgen(structural, method, getter, js_name = currentTarget)]
    pub fn current_target(this: &Event) -> JsValue;
    #[wasm_bindgen(structural, method, getter, js_name = eventPhase)]
    pub fn event_phase(this: &Event) -> PhaseType;
    #[wasm_bindgen(structural, method, getter)]
    pub fn bubbles(this: &Event) -> bool;
    #[wasm_bindgen(structural, method, getter, js_name = timeStamp)]
    fn timestamp(this: &Event) -> DOMHighResTimeStamp;
    #[wasm_bindgen(structural, method, js_name = stopPropagation)]
    pub fn stop_propagation(this: &Event);
    #[wasm_bindgen(structural, method, js_name = preventDefault)]
    pub fn prevent_default(this: &Event);
}