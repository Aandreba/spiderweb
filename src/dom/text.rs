use std::{cell::{UnsafeCell}, borrow::Borrow, rc::{Rc, Weak}};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use crate::state::{Listener, StateCell};
use super::{DomNode, Component, Element, IntoComponent};

pub type StaticText = Text<()>;
pub type DisplayText<'a, T> = Text<DisplayFn<'a, T>>;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    #[wasm_bindgen(js_name = Text, extends = DomNode)]
    pub(super) type DomText;

    #[wasm_bindgen(constructor, js_class = "Text")]
    pub(super) fn new (v: &str) -> DomText;
    #[wasm_bindgen(structural, method, setter, js_class = "Text")]
    pub(super) fn set_data (this: &DomText, v: &str);
}

#[cfg(not(feature = "nightly"))]
pub type DisplayFn<'a, T> = Box<dyn 'a + Send + Sync + FnMut(&T) -> String>;

#[cfg(feature = "nightly")]
pub struct DisplayFn<'a, T: ?Sized> (std::marker::PhantomData<&'a T>);

#[cfg(feature = "nightly")]
impl<'a, T: 'a + ?Sized + ToString> FnMut<(&T,)> for DisplayFn<'a, T> {
    #[inline]
    extern "rust-call" fn call_mut (&mut self, (t,): (&T,)) -> Self::Output {
        T::to_string(t)
    }
}

#[cfg(feature = "nightly")]
impl<'a, T: 'a + ?Sized + ToString> FnOnce<(&T,)> for DisplayFn<'a, T> {
    type Output = String;

    #[inline]
    extern "rust-call" fn call_once(self, (t,): (&T,)) -> Self::Output {
        T::to_string(t)
    }
}

#[cfg(feature = "nightly")]
unsafe impl<T> Send for DisplayFn<'_, T> {}
#[cfg(feature = "nightly")]
unsafe impl<T> Sync for DisplayFn<'_, T> {}

pub struct Text<F: ?Sized> {
    inner: DomText,
    f: UnsafeCell<F>
}

impl StaticText {
    #[inline]
    pub fn constant (s: &str) -> Self {
        return Self {
            inner: DomText::new(s),
            f: UnsafeCell::new(())
        }
    }
}

impl<F> Text<F> {
    #[inline]
    pub fn new<'a, T, S> (parent: &StateCell<'a, T>, mut f: F) -> Rc<Self>
    where
        T: ?Sized,
        S: Borrow<str>,
        F: 'a + FnMut(&T) -> S
    {
        let this = Rc::new(Self {
            inner: parent.with(|x| DomText::new(f(x).borrow())),
            f: UnsafeCell::new(f)
        });

        parent.bind_weak(Rc::downgrade(&this) as Weak<dyn Listener<T>>);
        return this
    }
}

impl<'a, T: 'a + ?Sized + ToString> Text<DisplayFn<'a, T>> {
    #[inline]
    pub fn display (parent: &StateCell<'a, T>) -> Rc<Self> where T: 'a {
        Self::new(
            parent,
            #[cfg(not(feature = "nightly"))]
            Box::new(ToString::to_string),
            #[cfg(feature = "nightly")]
            DisplayFn(std::marker::PhantomData),
        )
    }
}

impl IntoComponent for &str {
    type Component = StaticText;
    type State = ();

    #[inline]
    fn into_component (self) -> Self::Component {
        StaticText::constant(self)
    }
}

impl IntoComponent for String {
    type Component = StaticText;
    type State = ();

    #[inline]
    fn into_component (self) -> Self::Component {
        StaticText::constant(&self)
    }
}

impl IntoComponent for Box<str> {
    type Component = StaticText;
    type State = ();

    #[inline]
    fn into_component (self) -> Self::Component {
        StaticText::constant(&self)
    }
}

impl IntoComponent for Rc<str> {
    type Component = StaticText;
    type State = ();

    #[inline]
    fn into_component (self) -> Self::Component {
        StaticText::constant(&self)
    }
}

impl Component for StaticText {
    type State = ();

    #[inline]
    fn render (self) -> Result<Element<Self::State>, JsValue> {
        return Ok(Element::from_dom(self.inner.into(), ()))
    }
}

impl<F: 'static> Component for Rc<Text<F>> {
    type State = Self;

    #[inline]
    fn render (self) -> Result<Element<Self::State>, JsValue> {
        return Ok(Element::from_dom(self.inner.clone().into(), self))
    }
}

impl<T: ?Sized, S: Borrow<str>, F: ?Sized + FnMut(&T) -> S> Listener<T> for Text<F> {
    #[inline]
    fn receive (&self, v: &T) {
        unsafe {
            let s = (&mut *self.f.get())(v);
            self.inner.set_data(s.borrow())
        }
    }
}