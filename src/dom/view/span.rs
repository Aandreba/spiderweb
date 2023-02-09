use super::{Text, TextAlignment};
use crate::{
    dom::{Component, DomHtmlElement, Element, IntoComponent},
    state::StateCell,
    WeakRef,
};
use std::borrow::Borrow;
use wasm_bindgen::{JsCast, JsValue};

pub struct Span {
    inner: Element<()>,
}

impl Span {
    #[inline]
    pub fn constant(s: &str) -> Result<Self, JsValue> {
        Self::constant_aligned(s, TextAlignment::default())
    }

    pub fn constant_aligned(s: &str, align: impl Into<TextAlignment>) -> Result<Self, JsValue> {
        let inner = Element::new("span", ());
        unsafe { inner.inner().style().set_property("text-align", align.into().to_str())? };

        let text = Text::new(s);
        unsafe { inner.inner().append_child(&text)? };
        return Ok(Self { inner });
    }
}

impl Span {
    #[inline]
    pub fn dynamic<'a, T, F, S>(state: &StateCell<'a, T>, f: F) -> Result<Self, JsValue>
    where
        T: ?Sized,
        F: 'a + FnMut(&T) -> S,
        S: Borrow<str>,
    {
        Self::dynamic_aligned(state, f, TextAlignment::default())
    }

    pub fn dynamic_aligned<'a, T, F, S>(state: &StateCell<'a, T>, mut f: F, align: impl Into<TextAlignment>) -> Result<Self, JsValue>
    where
        T: ?Sized,
        F: 'a + FnMut(&T) -> S,
        S: Borrow<str>,
    {
        let this = Self::constant_aligned(state.with(&mut f).borrow(), align)?;
        let inner = unsafe { WeakRef::new(this.inner.inner())? };

        state.register_weak(move |x| {
            if let Some(inner) = inner.deref() {
                let inner = unsafe {
                    inner
                        .unchecked_ref::<DomHtmlElement>()
                        .first_child()
                        .unwrap_unchecked()
                        .unchecked_into::<Text>()
                };

                inner.set_data(f(x).borrow());
                return true;
            }
            return false;
        });

        return Ok(this)
    }
}

impl Span {
    #[inline]
    pub fn fmt<T: ?Sized + ToString> (t: &T) -> Result<Self, JsValue> {
        Self::fmt_aligned(t, TextAlignment::default())
    }

    #[inline]
    pub fn fmt_aligned<T: ?Sized + ToString> (t: &T, align: impl Into<TextAlignment>) -> Result<Self, JsValue> {
        return Self::constant_aligned(ToString::to_string(t).as_str(), align)
    }
}

impl Span {
    #[inline]
    pub fn display<'a, T: 'a + ?Sized + ToString> (state: &StateCell<'a, T>) -> Result<Self, JsValue> {
        return Self::display_aligned(state, TextAlignment::default())
    }

    #[inline]
    pub fn display_aligned<'a, T: 'a + ?Sized + ToString> (state: &StateCell<'a, T>, align: impl Into<TextAlignment>) -> Result<Self, JsValue> {
        return Self::dynamic_aligned(state, ToString::to_string, align)
    }
}

impl Component for Span {
    type State = ();

    #[inline]
    fn render(self) -> Result<Element<Self::State>, JsValue> {
        Ok(self.inner)
    }
}

impl IntoComponent for &str {
    type Component = Result<Span, JsValue>;
    type State = ();

    #[inline]
    fn into_component (self) -> Self::Component {
        Span::constant(self)
    }
}