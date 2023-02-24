use wasm_bindgen::JsValue;
use super::element::{Element, ElementRef};
use std::{cell::UnsafeCell, pin::Pin};

pub struct Component<T: ?Sized> {
    pub(super) element: Element,
    state: UnsafeCell<T>,
}

pub struct ComponentChild<'e, 's, T: ?Sized> {
    element: ElementRef<'e>,
    state: Pin<&'s UnsafeCell<T>>,
}

impl<T> Component<T> {
    #[inline]
    pub fn new(tag: &str, state: T) -> Self {
        return Self {
            element: Element::new(tag),
            state: UnsafeCell::new(state),
        };
    }

    #[inline]
    pub fn append_child_pinned (self: Pin<&Self>, element: Element) -> Result<ComponentChild<'_, '_, T>, JsValue> { 
        unsafe { self.get_ref().append_child_unchecked(element) }
    }
   
    #[inline]
    pub fn append_child (&self, element: Element) -> Result<ComponentChild<'_, '_, T>, JsValue> where T: Unpin { 
        unsafe { self.append_child_unchecked(element) }
    }
    
    #[inline]
    pub unsafe fn append_child_unchecked (&self, element: Element) -> Result<ComponentChild<'_, '_, T>, JsValue> { 
        let element = self.element.append_child(element)?;
        return Ok(ComponentChild { element, state: Pin::new_unchecked(&self.state) })
    }

    #[inline]
    pub fn add_event_listener<F: FnMut(&mut T)>(self: Pin<&Self>, event: &'static str, mut f: F) {
        let f = move || unsafe {
            f(&mut *self.state.get());
        };

        let f = unsafe {
            core::mem::transmute::<Box<dyn FnMut()>, Box<dyn 'static + FnMut()>>(Box::new(f))
        };
        let _handle = self.element.add_event_listener(event, f);
    }
}

impl<'e, 's, T> ComponentChild<'e, 's, T> {
    #[inline]
    pub fn append_child<'e1> (&'e1 self, element: Element) -> Result<ComponentChild<'e1, 's, T>, JsValue> where 'e: 'e1 { 
        let element = self.element.append_child(element)?;
        return Ok(ComponentChild { element, state: self.state })
    }

    #[inline]
    pub fn add_event_listener<F: FnMut(&mut T)>(&self, event: &'static str, mut f: F) where T: Unpin {
        let f = move || unsafe {
            f(&mut *self.state.get());
        };

        let f = unsafe {
            core::mem::transmute::<Box<dyn FnMut()>, Box<dyn 'static + FnMut()>>(Box::new(f))
        };
        let _handle = self.element.add_event_listener(event, f);
    }
    
    #[inline]
    pub fn add_event_listener_pinned<F: FnMut(Pin<&mut T>)>(&self, event: &'static str, mut f: F) {
        let f = move || unsafe {
            f(Pin::new_unchecked(&mut *self.state.get()));
        };

        let f = unsafe {
            core::mem::transmute::<Box<dyn FnMut()>, Box<dyn 'static + FnMut()>>(Box::new(f))
        };
        let _handle = self.element.add_event_listener(event, f);
    }
}