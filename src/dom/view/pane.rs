use std::ops::AddAssign;
use wasm_bindgen::{JsValue, JsCast, prelude::wasm_bindgen};
use crate::{dom::{IntoComponent, Component, Element, DomHtmlElement}, state::StateCell, WeakRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    Vertical,
    Horizontal
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alignment {
    Start = "flex-start",
    Center = "center",
    End = "flex-end"
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct ChildState<T> {
    pub inner: T,
    size: f32
}

pub struct Pane {
    inner: Element<StateCell<'static, f32>>,
    orientation: Orientation
}

impl Pane {
    pub fn new (orient: Orientation, hoz: Alignment, vert: Alignment) -> Result<Self, JsValue> {
        let inner = Element::new("div", StateCell::new(0));

        let style = inner.inner.unchecked_ref::<DomHtmlElement>().style();
        style.set_property("style", "flex")?;
        style.set_property("justify-content", hoz.to_str())?;
        style.set_property("align-items", vert.to_str())?;
        
        todo!()
    }

    pub fn push<T: IntoComponent> (&self, child: T, size: f32) -> Result<(), JsValue> {
        let child = child.into_component().render()?;
        let child = Element {
            inner: child.inner,
            current_id: child.current_id,
            children: child.children,
            state: ChildState { inner: child.state, size },
        };

        if let Some(child) = child.inner.dyn_ref::<DomHtmlElement>() {
            let style = child.style();
            let child = WeakRef::new(child)?;

            match self.orientation {
                Orientation::Horizontal => {
                    style.set_property("height", "100%")?;
                    self.inner.state().register_weak(move |sum| {
                        if let Some(inner) = child.deref() {
                            if let Err(e) = inner.unchecked_ref::<DomHtmlElement>().style().set_property("width", &format!("calc(100% * {} / {})", size, sum)) {
                                crate::macros::eprintln!(&e)
                            }
                            return true
                        }
                        return false
                    });
                },
                Orientation::Vertical => {
                    style.set_property("width", "100%")?;
                    self.inner.state().register_weak(move |sum| {
                        if let Some(inner) = child.deref() {
                            if let Err(e) = inner.unchecked_ref::<DomHtmlElement>().style().set_property("height", &format!("calc(100% * {} / {})", size, sum)) {
                                crate::macros::eprintln!(&e)
                            }
                            return true
                        }
                        return false
                    });
                }
            }
        } else if let Some(child) = child.inner.dyn_ref::<DomText>() {
            todo!()
        }

        self.inner.state().update(|sum| sum.add_assign(size));

        todo!()
    }
}

impl Component for Pane {
    type State = StateCell<'static, f32>;

    #[inline]
    fn render (self) -> Result<crate::dom::Element<Self::State>, wasm_bindgen::JsValue> {
        Ok(self.inner)
    }
}