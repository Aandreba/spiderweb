use std::borrow::Cow;
use js_sys::Function;
use wasm_bindgen::{prelude::Closure, JsCast};
use crate::dom::{Component, Element, Text};

pub struct Button {
    name: Cow<'static, str>,
    onclick: Closure<dyn FnMut()>
}

impl Button {
    #[inline]
    pub fn new<F: 'static + FnMut()> (name: impl Into<Cow<'static, str>>, onclick: F) -> Self {
        return Self {
            name: name.into(),
            onclick: Closure::new(onclick)
        }
    }
}

impl Component for Button {
    type State = Closure<dyn FnMut()>;

    fn render (self) -> Result<Element<Self::State>, wasm_bindgen::JsValue> {
        let element = Element::new("button", self.onclick);
        element.append_child(Text::new_static(&self.name))?;
        element.set_callback_ref("onclick", |x| x.as_ref().unchecked_ref::<Function>());
        return Ok(element)
    }
}