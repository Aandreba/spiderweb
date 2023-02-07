use super::Element;
use std::{rc::Rc, any::Any};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    /// Raw DOM [Node](https://developer.mozilla.org/en-US/docs/Web/API/Node)
    #[derive(Debug, Clone)]
    #[wasm_bindgen(js_name = Node)]
    pub type DomNode;

    #[wasm_bindgen(structural, method, catch, js_name = appendChild)]
    pub fn append_child(this: &DomNode, node: &DomNode) -> Result<DomNode, JsValue>;
    #[wasm_bindgen(structural, method, catch, js_name = removeChild)]
    pub fn remove_child(this: &DomNode, node: &DomNode) -> Result<DomNode, JsValue>;
}

/// A type that can be added to the DOM
pub trait Component {
    type State: Any;
    fn render (self) -> Result<Element<Self::State>, JsValue>;
}

/// A type that can be added to the DOM
pub trait ComponentRef: Component {
    fn render(&self) -> Result<Element<Self::State>, JsValue>;
}

impl<T: ?Sized + ComponentRef> Component for &T {
    type State = T::State;

    #[inline]
    fn render(self) -> Result<Element<Self::State>, JsValue> {
        ComponentRef::render(self)
    }
}

impl<T: Component> Component for Box<T> {
    type State = T::State;

    #[inline]
    fn render(self) -> Result<Element<Self::State>, JsValue> {
        T::render(*self)
    }
}

impl<T: ?Sized + ComponentRef> ComponentRef for Rc<T> {
    #[inline]
    fn render(&self) -> Result<Element<Self::State>, JsValue> {
        <T as ComponentRef>::render(self)
    }
}

impl<T: ?Sized + ComponentRef> Component for Rc<T> {
    type State = T::State;

    #[inline]
    fn render(self) -> Result<Element<Self::State>, JsValue> {
        <T as ComponentRef>::render(&self)
    }
}