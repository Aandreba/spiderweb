use crate::{state::StateCell, WeakRef};

use super::{create_element, Component, DomNode, IntoComponent};
use js_sys::Function;
use slab::Slab;
use std::{
    any::Any,
    borrow::{Borrow, Cow},
    cell::{UnsafeCell},
    hint::unreachable_unchecked,
    marker::PhantomData,
    ops::Deref,
    rc::Rc,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue, UnwrapThrowExt};

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    #[wasm_bindgen(js_name = HTMLElement, extends = DomNode)]
    pub type DomHtmlElement;

    #[derive(Debug, Clone, PartialEq)]
    #[wasm_bindgen(js_name = CSSStyleDeclaration)]
    pub(super) type CssStyleDeclaration;

    #[wasm_bindgen(structural, method, js_name = getAttribute)]
    fn get_attribute(this: &DomHtmlElement, name: &str) -> String;

    #[wasm_bindgen(structural, method, catch, js_name = setAttribute)]
    fn set_attribute(this: &DomHtmlElement, name: &str, value: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(structural, method, js_name = cloneNode)]
    fn clone_node(this: &DomHtmlElement, deep: bool) -> DomHtmlElement;

    #[wasm_bindgen(structural, method, getter)]
    pub(super) fn style(this: &DomHtmlElement) -> CssStyleDeclaration;

    #[wasm_bindgen(structural, method, js_name = getPropertyValue)]
    pub(super) fn get_property(this: &CssStyleDeclaration, name: &str) -> String;
    #[wasm_bindgen(structural, method, catch, js_name = setProperty)]
    pub(super) fn set_property(
        this: &CssStyleDeclaration,
        name: &str,
        value: &str,
    ) -> Result<(), JsValue>;
}

pub type StatelessElement = Element<()>;
pub type ChildHandleRef<'a, T, S> = ChildHandle<T, &'a Element<S>>;
pub type ChildHandleShared<T, S> = ChildHandle<T, Rc<Element<S>>>;

pub struct ChildHandle<T, E> {
    id: usize,
    parent: E,
    _phtm: PhantomData<T>,
    _nothread: PhantomData<*mut ()>,
}

pub struct Element<T: ?Sized> {
    pub(super) inner: DomHtmlElement,
    // id's can only increase, thus list is always sorted. let's use binary search!
    pub(super) children: UnsafeCell<Slab<Element<Box<dyn Any>>>>,
    pub(super) state: T,
}

impl<T: ?Sized> Element<T> {
    #[inline]
    pub(super) fn from_dom(inner: DomHtmlElement, state: T) -> Self where T: Sized {
        return Self {
            inner,
            children: Default::default(),
            state,
        };
    }

    #[inline]
    pub fn new(tag: &str, state: T) -> Self where T: Sized {
        let inner = create_element(tag);
        return Self {
            inner: inner.into(),
            children: Default::default(),
            state,
        };
    }

    #[inline]
    pub fn default (tag: &str) -> Self where T: Default {
        Self::new(tag, Default::default())
    }

    #[inline]
    pub unsafe fn inner(&self) -> &DomHtmlElement {
        &self.inner
    }

    #[inline]
    pub fn state(&self) -> &T {
        return &self.state;
    }

    #[inline]
    pub fn set_attribute(&self, name: &str, value: &str) -> Result<(), JsValue> {
        if let Some(inner) = self.inner.dyn_ref::<DomHtmlElement>() {
            return inner.set_attribute(name, value);
        } else {
            return Err(JsValue::from_str("This element's node isn't a DOM element"));
        }
    }

    pub fn set_dyn_attribute<'a, F, S, U>(
        &self,
        name: &'a str,
        state: &StateCell<'a, U>,
        mut f: F,
    ) -> Result<(), JsValue>
    where
        U: ?Sized,
        F: 'a + FnMut(&U) -> S,
        S: Borrow<str>,
    {
        if let Some(inner) = self.inner.dyn_ref::<DomHtmlElement>() {
            let inner = WeakRef::new(inner)?;
            state.register_weak(move |x| {
                if let Some(inner) = inner.deref() {
                    if let Err(e) = inner
                        .unchecked_ref::<DomHtmlElement>()
                        .set_attribute(name, f(x).borrow())
                    {
                        crate::eprintln!(&e)
                    }
                }
                return false;
            });

            todo!()
        } else {
            return Err(JsValue::from_str("This element's node isn't a DOM element"));
        }
    }

    #[inline]
    pub fn set_callback<F: for<'a> FnOnce(&'a T) -> Cow<'a, Function>>(&self, event: &str, f: F) {
        let f = f(&self.state);
        self.inner.add_event_listener(event, &f);
    }

    #[inline]
    pub fn set_callback_ref<F: for<'a> FnOnce(&'a T) -> &'a Function>(&self, event: &str, f: F) {
        self.set_callback(event, |x| Cow::Borrowed(f(x)))
    }

    #[inline]
    pub fn append_child<C: IntoComponent>(
        &self,
        child: C,
    ) -> Result<ChildHandleRef<'_, C::State, T>, JsValue> {
        Self::append_child_by_deref(self, child)
    }

    #[inline]
    pub fn append_child_shared<C: IntoComponent>(
        self: Rc<Self>,
        child: C,
    ) -> Result<ChildHandleShared<C::State, T>, JsValue> {
        Self::append_child_by_deref(self, child)
    }

    pub fn append_child_by_deref<D: Deref<Target = Self>, C: IntoComponent>(
        this: D,
        child: C,
    ) -> Result<ChildHandle<C::State, D>, JsValue> {
        let child = child.into_component().render()?;
        this.inner.append_child(&child.inner)?;
        let id = unsafe { &mut *this.children.get() }.insert(Element {
            inner: child.inner,
            children: child.children,
            state: Box::new(child.state),
        });

        return Ok(ChildHandle {
            id,
            parent: this,
            _phtm: PhantomData,
            _nothread: PhantomData,
        });
    }

    #[doc(hidden)]
    #[inline]
    pub fn append_child_inner<C: IntoComponent>(self, child: C) -> Result<Self, JsValue> where T: Sized {
        Self::append_child_by_deref(&self, child)?;
        return Ok(self);
    }
}

impl StatelessElement {
    #[inline]
    pub fn stateless (tag: &str) -> Self {
        Self::new(tag, ())
    }
}

impl<T: Any, S: ?Sized, E: Deref<Target = Element<S>>> ChildHandle<T, E> {
    /// Returns a reference to the child's state
    #[inline]
    pub fn state(&self) -> &T {
        unsafe {
            return match (&*self.parent.children.get()).get(self.id) {
                Some(x) => &*(x.state() as *const dyn Any as *const T),
                None => unreachable_unchecked(),
            };
        }
    }

    /// Detaches the child from it's parent, returning the child's state
    pub fn detach(self) -> Element<T> {
        unsafe {
            let children = &mut *self.parent.children.get();
            match children.try_remove(self.id) {
                Some(element) => {
                    let _ = self
                        .parent
                        .inner
                        .remove_child(&element.inner)
                        .unwrap_throw();

                    return Element {
                        inner: element.inner,
                        children: element.children,
                        #[cfg(feature = "nightly")]
                        state: *element.state.downcast_unchecked::<T>(),
                        #[cfg(not(feature = "nightly"))]
                        state: *element.state.downcast::<T>().unwrap_unchecked(),
                    };
                }
                None => unreachable_unchecked(),
            };
        }
    }
}

impl<T, S: ?Sized, E: Deref<Target = Element<S>>> Deref for ChildHandle<T, E> {
    type Target = Element<Box<dyn Any>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            match (&*self.parent.children.get()).get(self.id) {
                Some(x) => x,
                None => unreachable_unchecked(),
            }
        }
    }
}

impl<T: Clone> Clone for Element<T> {
    /// NOTE THAT CHILDREN AND EVENT LISTENERS WILL NOT BE CLONED
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone_node(false),
            children: UnsafeCell::new(Slab::new()),
            state: self.state.clone(),
        }
    }
}

impl<T: Any> Component for Element<T> {
    type State = T;

    #[inline]
    fn render(self) -> Result<Element<Self::State>, JsValue> {
        Ok(self)
    }
}
