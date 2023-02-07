use std::borrow::Cow;
use wasm_bindgen::prelude::Closure;
use crate::dom::{Component, Element};

pub struct Button {
    onclick: Closure<dyn FnMut()>
}

impl Button {
    #[inline]
    pub fn new<F: 'static + FnMut()> (name: impl Into<Cow<'static, str>>, onclick: F) -> Self {
        return Self {
            onclick: Closure::new(onclick)
        }
    }
}

impl Component for Button {
    type State = Closure<dyn FnMut()>;

    #[inline]
    fn render (self) -> Result<crate::dom::Element<Self::State>, wasm_bindgen::JsValue> {
        let element = Element::new("", state);
    }
}