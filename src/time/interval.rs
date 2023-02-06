use std::{marker::PhantomData, time::Duration, mem::ManuallyDrop};
use futures::{Stream, StreamExt};
use wasm_bindgen::{prelude::{wasm_bindgen, Closure}, JsValue};
use crate::channel::mpsc::{Receiver, channel};
use super::timeout2millis;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setInterval)]
    pub fn set_interval (handler: &JsValue, millis: i32) -> f64;
    #[wasm_bindgen(js_name = clearInterval)]
    pub fn clear_interval (id: f64);
}

/// Handler of a JavaScript interval
/// 
/// Once every specified time delay, the handle's callback will be called, and a new value is sent to the stream.
/// 
/// When dropped, the interval will be automatically cleared and it's underlying channel closed.
#[derive(Debug)]
pub struct Interval<'a, T: 'a> {
    id: f64,
    recv: Receiver<T>,
    _closure: Closure<dyn 'static + FnMut()>,
    _phtm: PhantomData<Closure<dyn 'a + FnMut() -> T>>
}

impl<'a, T> Interval<'a, T> {
    /// Creates a new interval that executes `f` with the specified time delay of `timeout`. 
    pub fn new<F: 'a + FnMut() -> T> (mut f: F, timeout: Duration) -> Self {
        let (send, recv) = channel::<T>();
        let mut send = Some(send);

        let closure = Box::new(move || {
            let v = f();
            if let Some(ref inner_send) = send {
                if inner_send.try_send(v).is_err() {
                    send = None
                }
            }
        });

        let closure = unsafe {
            core::mem::transmute::<Box<dyn 'a + FnMut()>, Box<dyn 'static + FnMut()>>(closure)
        };
        
        let closure = Closure::wrap(closure);
        let id = set_interval(closure.as_ref(), timeout2millis(timeout));
        
        return Self {
            id,
            _closure: closure,
            recv,
            _phtm: PhantomData
        }
    }
}

impl<'a, T> Interval<'a, T> {
    /// Returns the id of the interval.
    /// 
    /// # Safety
    /// This handler must be forgoten, or this id must not be used to clear the interval manually.
    /// Both things ocurring is considered undefined bahaviour.
    #[inline]
    pub unsafe fn id (&self) -> f64 {
        self.id
    }

    /// Converts the handler into its raw components.
    /// 
    /// After calling this method, the caller is responsable for dropping the closure and clearing the
    /// interval.
    #[inline]
    pub fn into_raw_parts (self) -> (f64, Receiver<T>, Closure<dyn 'static + FnMut()>) {
        unsafe {
            let this = ManuallyDrop::new(self);
            (this.id, core::ptr::read(&this.recv), core::ptr::read(&this._closure))
        }
    }

    /// Creates a handle from its raw components.
    /// 
    /// # Safety
    /// 
    #[inline]
    pub unsafe fn from_raw_parts (id: f64, recv: Receiver<T>, closure: Closure<dyn 'static + FnMut()>) -> Self {
        return Self {
            id,
            recv,
            _closure: closure,
            _phtm: PhantomData,
        }
    }
}

impl<T> Interval<'static, T> {
    #[inline]
    pub fn leak (self) -> (f64, Receiver<T>) {
        let (id, recv, _) = self.into_raw_parts();
        (id, recv)
    }
}

impl<T> Stream for Interval<'_, T> {
    type Item = T;

    #[inline]
    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        self.recv.poll_next_unpin(cx)
    }
}

impl<T> Drop for Interval<'_, T> {
    #[inline]
    fn drop(&mut self) {
        clear_interval(self.id)
    }
}