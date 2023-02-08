use crate::{state::StateCell, WeakRef};

use super::{create_element, Component, DomNode, IntoComponent};
use js_sys::Function;
use std::{
    any::Any,
    borrow::{Borrow, Cow},
    cell::{Cell, UnsafeCell},
    collections::VecDeque,
    hint::unreachable_unchecked,
    marker::PhantomData,
    num::NonZeroU64,
    ops::Deref,
    pin::Pin,
    rc::Rc,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue, UnwrapThrowExt};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = HTMLElement, extends = DomNode)]
    pub(super) type DomHtmlElement;

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

pub type ChildHandleRef<'a, T> = ChildHandle<&'a Element<T>>;
pub type ChildHandleShared<T> = ChildHandle<Rc<Element<T>>>;

pub struct ChildHandle<E> {
    id: NonZeroU64,
    parent: E,
    _phtm: PhantomData<*mut ()>,
}

pub struct Element<T: ?Sized> {
    pub(super) inner: DomNode,
    pub(super) current_id: Cell<NonZeroU64>,
    // id's can only increase, thus list is always sorted. let's use binary search!
    pub(super) children: UnsafeCell<VecDeque<(NonZeroU64, Element<Pin<Box<dyn Any>>>)>>,
    pub(super) state: T,
}

impl<T> Element<T> {
    #[inline]
    pub(super) fn from_dom(inner: DomNode, state: T) -> Self {
        return Self {
            inner,
            current_id: unsafe { Cell::new(NonZeroU64::new_unchecked(1)) },
            children: Default::default(),
            state,
        };
    }

    #[inline]
    pub fn new(tag: &str, state: T) -> Self {
        let inner = create_element(tag);
        return Self {
            inner: inner.into(),
            current_id: unsafe { Cell::new(NonZeroU64::new_unchecked(1)) },
            children: Default::default(),
            state,
        };
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
                        crate::macros::eprintln!(&e)
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
    ) -> Result<ChildHandleRef<'_, T>, JsValue> {
        Self::append_child_by_deref(self, child)
    }

    #[inline]
    pub fn append_child_shared<C: IntoComponent>(
        self: Rc<Self>,
        child: C,
    ) -> Result<ChildHandleShared<T>, JsValue> {
        Self::append_child_by_deref(self, child)
    }

    pub fn append_child_by_deref<D: Deref<Target = Self>, C: IntoComponent>(
        this: D,
        child: C,
    ) -> Result<ChildHandle<D>, JsValue> {
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
        unsafe { &mut *this.children.get() }.push_back((
            id,
            Element {
                inner: child.inner,
                current_id: child.current_id,
                children: child.children,
                state: Box::pin(child.state),
            },
        ));

        return Ok(ChildHandle {
            id,
            parent: this,
            _phtm: PhantomData,
        });
    }

    #[doc(hidden)]
    #[inline]
    pub fn append_child_inner<C: IntoComponent>(self, child: C) -> Result<Self, JsValue> {
        Self::append_child_by_deref(&self, child)?;
        return Ok(self);
    }
}

impl Element<Box<dyn Any>> {
    #[inline]
    pub fn downcast<T: Any>(self) -> Result<Element<T>, Self> {
        match self.state.downcast::<T>() {
            Ok(state) => Ok(Element {
                inner: self.inner,
                current_id: self.current_id,
                children: self.children,
                state: *state,
            }),
            Err(state) => Err(Self { state, ..self }),
        }
    }

    #[inline]
    pub fn downcast_boxed<T: Any>(self) -> Result<Element<Box<T>>, Self> {
        match self.state.downcast::<T>() {
            Ok(state) => Ok(Element {
                inner: self.inner,
                current_id: self.current_id,
                children: self.children,
                state,
            }),
            Err(state) => Err(Self { state, ..self }),
        }
    }
}

impl Element<Pin<Box<dyn Any>>> {
    #[inline]
    pub fn downcast<T: Unpin + Any>(self) -> Result<Element<T>, Self> {
        let state = unsafe { Pin::into_inner_unchecked(self.state) };
        match state.downcast::<T>() {
            Ok(state) => Ok(Element {
                inner: self.inner,
                current_id: self.current_id,
                children: self.children,
                state: *state,
            }),
            Err(state) => Err(Self {
                state: Box::into_pin(state),
                ..self
            }),
        }
    }

    #[inline]
    pub fn downcast_unpin<T: Unpin + Any>(self) -> Result<Element<Box<T>>, Self> {
        let state = unsafe { Pin::into_inner_unchecked(self.state) };
        match state.downcast::<T>() {
            Ok(state) => Ok(Element {
                inner: self.inner,
                current_id: self.current_id,
                children: self.children,
                state,
            }),
            Err(state) => Err(Self {
                state: Box::into_pin(state),
                ..self
            }),
        }
    }
    
    #[inline]
    pub fn downcast_boxed<T: Any>(self) -> Result<Element<Pin<Box<T>>>, Self> {
        let state = unsafe { Pin::into_inner_unchecked(self.state) };
        match state.downcast::<T>() {
            Ok(state) => Ok(Element {
                inner: self.inner,
                current_id: self.current_id,
                children: self.children,
                state: Box::into_pin(state),
            }),
            Err(state) => Err(Self {
                state: Box::into_pin(state),
                ..self
            }),
        }
    }
}

impl<T: ?Sized, E: Deref<Target = Element<T>>> ChildHandle<E> {
    /// Detaches the child from it's parent, returning the child's state
    pub fn detach(self) -> Element<Pin<Box<dyn Any>>> {
        unsafe {
            let children = &mut *self.parent.children.get();
            match children.binary_search_by(|(x, _)| x.cmp(&self.id)) {
                Ok(x) => {
                    let element = children.remove(x).unwrap_unchecked().1;
                    let _ = self
                        .parent
                        .inner
                        .remove_child(&element.inner)
                        .unwrap_throw();
                    return element;
                }
                Err(_) => unreachable_unchecked(),
            };
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
