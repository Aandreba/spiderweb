use super::{create_element, Component, HtmlElement};
use std::{any::Any, cell::{UnsafeCell, Cell}, num::NonZeroU64, ops::Deref, rc::{Rc}, marker::PhantomData, hint::unreachable_unchecked, borrow::Cow};
use js_sys::Function;
use wasm_bindgen::JsValue;

pub type ChildHandleRef<'a, T> = ChildHandle<&'a Element<T>>;
pub type ChildHandleShared<T> = ChildHandle<Rc<Element<T>>>;

pub struct ChildHandle<E> {
    id: NonZeroU64,
    parent: E,
    _phtm: PhantomData<*mut ()>
}

pub struct Element<T: ?Sized> {
    inner: HtmlElement,
    current_id: Cell<NonZeroU64>,
    // id's can only increase, thus list is always sorted. let's use binary search!
    children: UnsafeCell<Vec<(NonZeroU64, Box<dyn Any>)>>,
    state: T,
}

impl<T> Element<T> {
    #[inline]
    pub(super) fn from_dom(inner: HtmlElement, state: T) -> Self {
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
            inner,
            current_id: unsafe { Cell::new(NonZeroU64::new_unchecked(1)) },
            children: Default::default(),
            state,
        };
    }

    #[inline]
    pub fn set_callback<F: for<'a> FnOnce(&'a T) -> Cow<'a, Function>> (&self, event: &str, f: F) {
        let f = f(&self.state);
    }

    #[inline]
    pub fn append_child<C: Component> (&self, child: C) -> Result<ChildHandleRef<'_, T>, JsValue> {
        Self::append_child_by_deref(self, child)
    }

    #[inline]
    pub fn append_child_shared<C: Component> (self: Rc<Self>, child: C) -> Result<ChildHandleShared<T>, JsValue> {
        Self::append_child_by_deref(self, child)
    }

    pub fn append_child_by_deref<D: Deref<Target = Self>, C: Component> (this: D, child: C) -> Result<ChildHandle<D>, JsValue> {
        let child = child.render()?;
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
        unsafe { &mut *this.children.get() }.push((id, Box::new(child.state)));
        return Ok(ChildHandle { id, parent: this, _phtm: PhantomData })
    }
}

impl<T: ?Sized, E: Deref<Target = Element<T>>> ChildHandle<E> {
    /// Detaches the child from it's parent, returning the child's state
    #[inline]
    pub fn detach (self) -> Box<dyn Any> {
        let children = unsafe { &mut *self.parent.children.get_mut() };
        return match children.binary_search_by(|(x, _)| x.cmp(&self.id)) {
            Ok(x) => children.remove(x).1,
            Err(_) => unsafe { unreachable_unchecked() }
        }
    }
}

impl<T: Any> Component for Element<T> {
    type State = T;

    #[inline]
    fn render (self) -> Result<Element<Self::State>, JsValue> {
        Ok(self)
    }
}
