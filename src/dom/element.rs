use wasm_bindgen::JsValue;
use super::{Node, HtmlElement, create_element, IntoNode};

pub struct Element {
    inner: HtmlElement
}

impl Element {
    pub fn new (tag: &str, children: impl IntoIterator<Item = impl IntoNode>) -> Result<Self, JsValue> {
        let inner = create_element(tag);
        for child in children {
            child.into_node().append_to(&inner)?;
        }

        return Ok(Self { inner })
    }
}

impl Node for Element {
    #[inline]
    fn append_to (&self, node: &super::DomNode) -> Result<(), wasm_bindgen::JsValue> {
        self.inner.append_to(node)
    }
}