use std::rc::Rc;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    /// Raw DOM [Node](https://developer.mozilla.org/en-US/docs/Web/API/Node)
    #[derive(Debug, Clone)]
    #[wasm_bindgen(js_name = Node)]
    pub type DomNode;

    #[wasm_bindgen(structural, method, catch, js_name = appendChild)]
    pub fn append_child (this: &DomNode, node: &DomNode) -> Result<DomNode, JsValue>;
}

/// A type that can be converted into a [`Node`]
pub trait IntoNode {
    type Node: Node;

    fn into_node (self) -> Self::Node;
}

/// A type that can be added to the DOM
pub trait Node {
    fn append_to (&self, node: &DomNode) -> Result<(), JsValue>;
}

impl Node for DomNode {
    #[inline]
    fn append_to (&self, node: &DomNode) -> Result<(), JsValue> {
        let _ = DomNode::append_child(node, self)?;
        return Ok(())
    }
}

impl<T: ?Sized + Node> Node for &T {
    #[inline]
    fn append_to (&self, node: &DomNode) -> Result<(), JsValue> {
        T::append_to(self, node)
    }
}

impl<T: Node> Node for Box<T> {
    #[inline]
    fn append_to (&self, node: &DomNode) -> Result<(), JsValue> {
        T::append_to(self, node)
    }
}

impl<T: ?Sized + Node> Node for Rc<T> {
    #[inline]
    fn append_to (&self, node: &DomNode) -> Result<(), JsValue> {
        T::append_to(self, node)
    }
}

impl<T: Node> IntoNode for T {
    type Node = Self;

    #[inline]
    fn into_node (self) -> Self::Node {
        self
    }
}