use std::ops::AddAssign;
use crate::{dom::{IntoComponent, Component, Element}, state::StateCell};

pub enum Orientation {
    Vertical,
    Horizontal
}

pub struct Pane {
    inner: Element<()>,
    elements: StateCell<'static, usize>,
    orientation: Orientation
}

impl Pane {
    #[inline]
    pub fn append<T: IntoComponent> (&self, size: f32) {
        self.elements.update(|x| x.add_assign(1));
    }
}

impl Component for Pane {
    type State = ();

    #[inline]
    fn render (self) -> Result<crate::dom::Element<Self::State>, wasm_bindgen::JsValue> {
        todo!()
    }
}