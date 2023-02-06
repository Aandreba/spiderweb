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

#[derive(Debug)]
pub struct Interval<'a, T: 'a> {
    id: f64,
    recv: Receiver<T>,
    _closure: Closure<dyn 'static + FnMut()>,
    _phtm: PhantomData<Closure<dyn 'a + FnMut() -> T>>
}

impl<'a, T> Interval<'a, T> {
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
    #[inline]
    pub unsafe fn id (&self) -> f64 {
        self.id
    }

    #[inline]
    pub unsafe fn into_raw_parts (self) -> (f64, Receiver<T>, Closure<dyn 'static + FnMut()>) {
        unsafe {
            let this = ManuallyDrop::new(self);
            (this.id, core::ptr::read(&this.recv), core::ptr::read(&this._closure))
        }
    }

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
        unsafe {
            let (id, recv, _) = self.into_raw_parts();
            (id, recv)
        }
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