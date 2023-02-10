use crate::{state::StateCell, WeakRef};

use super::{create_element, Component, DomNode, IntoComponent};
use js_sys::Function;
use vector_mapp::binary::BinaryMap;
use std::{
    any::Any,
    borrow::{Borrow, Cow},
    cell::{Cell, UnsafeCell},
    hint::unreachable_unchecked,
    marker::PhantomData,
    num::NonZeroU64,
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

pub type ChildHandleRef<'a, T, S> = ChildHandle<T, &'a Element<S>>;
pub type ChildHandleShared<T, S> = ChildHandle<T, Rc<Element<S>>>;

pub struct ChildHandle<T, E> {
    id: NonZeroU64,
    parent: E,
    _phtm: PhantomData<T>,
    _nothread: PhantomData<*mut ()>,
}

pub struct Element<T: ?Sized> {
    pub(super) inner: DomHtmlElement,
    pub(super) current_id: Cell<NonZeroU64>,
    // id's can only increase, thus list is always sorted. let's use binary search!
    pub(super)  children: UnsafeCell<BinaryMap<NonZeroU64, Element<Box<dyn Any>>>>,
    pub(super)  state: T,
}

impl<T> Element<T> {
    #[inline]
    pub(super) fn from_dom(inner: DomHtmlElement, state: T) -> Self {
        return Self {
            inner,
            current_id: unsafe { Cell::new(NonZeroU64::new_unchecked(1)) },
            children: Default::default(),
            state,
        };
    }

    #[inline]
    pub fn new (tag: &str, state: T) -> Self {
        let inner = create_element(tag);
        return Self {
            inner: inner.into(),
            current_id: unsafe { Cell::new(NonZeroU64::new_unchecked(1)) },
            children: Default::default(),
            state,
        };
    }

    #[inline]
    pub unsafe fn inner (&self) -> &DomHtmlElement {
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

        let id = unsafe {
            let prev = this.current_id.get();
            #[cfg(feature = "nightly")]
            this.current_id.set(prev.unchecked_add(1));
            #[cfg(not(feature = "nightly"))]
            this.current_id.set(prev.checked_add(1).unwrap_unchecked());
            prev
        };

        // todo optimize in nightly
        unsafe {
            (&mut *this.children.get()).insert_back_unchecked(
                id,
                Element {
                    inner: child.inner,
                    current_id: child.current_id,
                    children: child.children,
                    state: Box::new(child.state),
                },
            );
        }

        return Ok(ChildHandle {
            id,
            parent: this,
            _phtm: PhantomData,
            _nothread: PhantomData
        });
    }

    #[doc(hidden)]
    #[inline]
    pub fn append_child_inner<C: IntoComponent>(self, child: C) -> Result<Self, JsValue> {
        Self::append_child_by_deref(&self, child)?;
        return Ok(self);
    }
}

impl<T: Any, S: ?Sized, E: Deref<Target = Element<S>>> ChildHandle<T, E> {
    /// Returns a reference to the child's state
    #[inline]
    pub fn state (&self) -> &T {
        unsafe {
            return match (&*self.parent.children.get()).get(&self.id) {
                Some(x) => &*(x.state() as *const dyn Any as *const T),
                None => unreachable_unchecked()
            }
        }
    }

    /// Detaches the child from it's parent, returning the child's state
    pub fn detach(self) -> Element<T> {
        unsafe {
            let children = &mut *self.parent.children.get();
            match children.remove(&self.id) {
                Some(element) => {
                    let _ = self
                        .parent
                        .inner
                        .remove_child(&element.inner)
                        .unwrap_throw();

                    return Element {
                        inner: element.inner,
                        current_id: element.current_id,
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
            match (&*self.parent.children.get()).get(&self.id) {
                Some(x) => x,
                None => unreachable_unchecked()
            }
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
