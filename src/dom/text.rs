use std::{cell::{UnsafeCell}, borrow::Borrow, rc::{Rc, Weak}};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use crate::state::{Subscriber, State};
use super::{JsNode, NodeRef};

pub type StaticText = Text<()>;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Text, extends = JsNode)]
    type DomText;

    #[wasm_bindgen(constructor, js_class = "Text")]
    fn new (v: &str) -> DomText;
    #[wasm_bindgen(structural, method, setter, js_class = "Text")]
    fn set_data (this: &DomText, v: &str);
}

#[cfg(not(feature = "nightly"))]
pub type DisplayFn<'a, T> = Box<dyn 'a + Send + Sync + FnMut(&T) -> String>;

#[cfg(feature = "nightly")]
pub struct DisplayFn<'a, T> (std::marker::PhantomData<&'a T>);

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
    pub fn new_static (s: &str) -> Self {
        return Self {
            inner: DomText::new(s),
            f: UnsafeCell::new(())
        }
    }
}

impl<F> Text<F> {
    #[inline]
    pub fn new<'a, T, S> (parent: &State<'a, T>, mut f: F) -> Rc<Self>
    where
        T: ?Sized,
        S: Borrow<str>,
        F: 'a + FnMut(&T) -> S
    {
        let this = Rc::new(Self {
            inner: parent.with(|x| DomText::new(f(x).borrow())),
            f: UnsafeCell::new(f)
        });

        parent.bind_weak(Rc::downgrade(&this) as Weak<dyn Subscriber<T>>);
        return this
    }
}

impl<'a, T: 'a + ?Sized + ToString> Text<DisplayFn<'a, T>> {
    #[inline]
    pub fn new_stringify (parent: &State<'a, T>) -> Rc<Self> where T: 'a {
        Self::new(
            parent,
            #[cfg(not(feature = "nightly"))]
            Box::new(ToString::to_string),
            #[cfg(feature = "nightly")]
            DisplayFn(std::marker::PhantomData),
        )
    }
}

impl<F: ?Sized> NodeRef for Text<F> {
    #[inline]
    fn append_to (&self, node: &JsNode) -> Result<(), JsValue> {
        node.append_child(&self.inner).map(|_| ())
    }
}

impl<T: ?Sized, S: Borrow<str>, F: ?Sized + FnMut(&T) -> S> Subscriber<T> for Text<F> {
    #[inline]
    fn update (&self, v: &T) {
        unsafe {
            let s = (&mut *self.f.get())(v);
            self.inner.set_data(s.borrow())
        }
    }
}