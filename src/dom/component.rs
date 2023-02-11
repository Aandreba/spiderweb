use super::{Element};
use std::{any::Any};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    /// Raw DOM [Node](https://developer.mozilla.org/en-US/docs/Web/API/Node)
    #[wasm_bindgen(js_name = Node, typescript_type = "Node")]
    #[derive(Debug, Clone, PartialEq)]
    pub type DomNode;

    #[wasm_bindgen(structural, method, getter, js_name = parentNode)]
    pub fn parent_node (this: &DomNode) -> Option<DomNode>;
    #[wasm_bindgen(structural, method, getter, js_name = firstChild)]
    pub fn first_child (this: &DomNode) -> Option<DomNode>;
    #[wasm_bindgen(structural, method, catch, js_name = appendChild)]
    pub fn append_child(this: &DomNode, node: &DomNode) -> Result<DomNode, JsValue>;
    #[wasm_bindgen(structural, method, catch, js_name = removeChild)]
    pub fn remove_child(this: &DomNode, node: &DomNode) -> Result<DomNode, JsValue>;
}

cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        /// Component without inherent state
        pub trait StatelessComponent = Component<State = ()>;
    } else {
        /// Component without inherent state
        pub trait StatelessComponent: Component<State = ()> {}
        impl<T: Component<State = ()>> StatelessComponent for T {}
    }
}

/// A type that can be added to the DOM
pub trait Component {
    type State: Any;

    fn render (self) -> Result<Element<Self::State>, JsValue>;
}

impl<T: Component> Component for Result<T, JsValue> {
    type State = T::State;

    #[inline]
    fn render (self) -> Result<Element<Self::State>, JsValue> {
        self.and_then(T::render)
    }
}

impl<T: Component> Component for Box<T> {
    type State = T::State;

    #[inline]
    fn render(self) -> Result<Element<Self::State>, JsValue> {
        T::render(*self)
    }
}

/// A type that can be converted into a [`Component`]
pub trait IntoComponent {
    type Component: Component<State = Self::State>;
    type State: Any;

    fn into_component (self) -> Self::Component;
}

impl<T: Component> IntoComponent for T {
    type Component = T;
    type State = <T as Component>::State;

    #[inline]
    fn into_component (self) -> Self::Component {
        self
    }
}