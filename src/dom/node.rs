use std::rc::Rc;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    #[wasm_bindgen(js_name = Node)]
    pub type JsNode;

    #[wasm_bindgen(structural, method, catch, js_name = appendChild)]
    pub fn append_child (this: &JsNode, node: &JsNode) -> Result<JsNode, JsValue>;
}

pub trait Node {
    fn append_to (self, node: &JsNode) -> Result<(), JsValue>;
}

pub trait NodeRef {
    fn append_to (&self, node: &JsNode) -> Result<(), JsValue>;
}

impl<T: Node> Node for Box<T> {
    #[inline]
    fn append_to (self, node: &JsNode) -> Result<(), JsValue> {
        T::append_to(*self, node)
    }
}

impl<T: ?Sized + NodeRef> Node for &T {
    #[inline]
    fn append_to (self, node: &JsNode) -> Result<(), JsValue> {
        <T as NodeRef>::append_to(self, node)
    }
}

impl<T: ?Sized + NodeRef> NodeRef for Rc<T> {
    #[inline]
    fn append_to (&self, node: &JsNode) -> Result<(), JsValue> {
        <T as NodeRef>::append_to(self, node)
    }
}