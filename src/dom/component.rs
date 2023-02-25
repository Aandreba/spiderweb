use super::element::{Element, MountedElement};
use std::{
    any::Any,
    cell::UnsafeCell,
    marker::{PhantomData},
    ops::Deref,
    pin::Pin, hint::unreachable_unchecked,
};
use pin_project::pin_project;
use wasm_bindgen::JsValue;

#[pin_project(!Unpin)]
pub struct Component<T: ?Sized> {
    pub(super) element: Element,
    #[pin] state: UnsafeCell<T>,
}

pub struct MountedComponent<'a, P: 'a, T: ?Sized> {
    pub(super) handle: MountedElement<P>,
    pub(super) _phtm: PhantomData<&'a T>,
}

pub struct ComponentChild<'e, 's, T: ?Sized> {
    element: MountedElement<&'e Element>,
    state: Pin<&'s UnsafeCell<T>>,
}

impl<T> Component<T> {
    #[inline]
    pub(super) fn new(tag: &str, state: T) -> Self {
        return Self {
            element: Element::new(tag),
            state: UnsafeCell::new(state)
        };
    }

    #[inline]
    pub fn append_child(
        self: Pin<&Self>,
        element: Element,
    ) -> Result<ComponentChild<'_, '_, T>, JsValue> {
        let this = self.project_ref();
        let element = this.element.append_child(element)?;
        return Ok(ComponentChild {
            element,
            state: this.state,
        });
    }

    #[inline]
    pub fn add_event_listener<F: FnMut(&mut T)>(self: Pin<&Self>, event: &'static str, mut f: F) where T: Unpin {
        let this = self.project_ref();

        let state = this.state;
        let f = move || unsafe {
            f(&mut *state.get());
        };

        let f = unsafe {
            core::mem::transmute::<Box<dyn FnMut()>, Box<dyn 'static + FnMut()>>(Box::new(f))
        };
        let _handle = this.element.add_event_listener(event, f);
    }

    #[inline]
    pub fn add_event_listener_pinned<F: FnMut(Pin<&mut T>)>(self: Pin<&Self>, event: &'static str, mut f: F) {
        let this = self.project_ref();

        let state = this.state;
        let f = move || unsafe {
            f(Pin::new_unchecked(&mut *Pin::into_inner_unchecked(state).get()));
        };

        let f = unsafe {
            core::mem::transmute::<Box<dyn FnMut()>, Box<dyn 'static + FnMut()>>(Box::new(f))
        };
        let _handle = this.element.add_event_listener(event, f);
    }
}

impl<'e, 's, T> ComponentChild<'e, 's, T> {
    #[inline]
    pub fn append_child<'e1>(
        &'e1 self,
        element: Element,
    ) -> Result<ComponentChild<'e1, 's, T>, JsValue>
    where
        'e: 'e1,
    {
        let element = self.element.append_child(element)?;
        return Ok(ComponentChild {
            element,
            state: self.state,
        });
    }

    #[inline]
    pub fn add_event_listener<F: FnMut(&mut T)>(&self, event: &'static str, mut f: F)
    where
        T: Unpin,
    {
        let state = self.state;
        let f = move || unsafe {
            f(&mut *state.get());
        };

        let f = unsafe {
            core::mem::transmute::<Box<dyn FnMut()>, Box<dyn 'static + FnMut()>>(Box::new(f))
        };

        let _handle = self.element.add_event_listener(event, f);
    }

    #[inline]
    pub fn add_event_listener_pinned<F: FnMut(Pin<&mut T>)>(&self, event: &'static str, mut f: F) {
        let state = self.state;
        let f = move || unsafe {
            f(Pin::new_unchecked(&mut *state.get()));
        };

        let f = unsafe {
            core::mem::transmute::<Box<dyn FnMut()>, Box<dyn 'static + FnMut()>>(Box::new(f))
        };
        let _handle = self.element.add_event_listener(event, f);
    }
}

impl<'a, P: 'a + Deref<Target = Element>, T> Deref for MountedComponent<'a, P, T> {
    type Target = Component<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            let inner = &*self.handle.parent.inner.get();
            match inner.children.get(self.handle.idx) {
                Some(super::element::Child::Component(x)) => {
                    return &*(x.as_ref().get_ref() as *const Component<dyn Any>
                        as *const Component<T>);
                }
                _ => unreachable_unchecked(),
            }
        }
    }
}
