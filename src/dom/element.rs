use wasm_bindgen::JsValue;
use super::{Node, HtmlElement, create_element, IntoNode};

#[derive(Debug, Clone)]
pub struct Element {
    inner: HtmlElement
}

impl Element {
    pub fn new (tag: &str) -> Self {
        let inner = create_element(tag);
        return Self { inner }
    }
    
    #[inline]
    pub fn append_child<T: IntoNode> (&self, child: T) -> Result<(), JsValue> {
        child.into_node().append_to(&self.inner)
    }

    #[doc(hidden)]
    #[inline]
    pub fn append_child_inner<T: IntoNode> (self, child: T) -> Result<Self, JsValue> {
        child.into_node().append_to(&self.inner)?;
        Ok(self)
    }
}

impl Node for Element {
    #[inline]
    fn append_to (&self, node: &super::DomNode) -> Result<(), wasm_bindgen::JsValue> {
        self.inner.append_to(node)
    }
}