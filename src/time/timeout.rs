use super::timeout2millis;
use crate::channel::oneshoot::{channel, Receiver};
use futures::{Future, FutureExt};
use std::{marker::PhantomData, mem::ManuallyDrop, pin::Pin, time::Duration};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsValue, UnwrapThrowExt,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    pub fn set_timeout(handler: &JsValue, millis: i32) -> f64;
    #[wasm_bindgen(js_name = clearTimeout)]
    pub fn clear_timeout(id: f64);
}

pub struct Timeout<'a, T: 'a> {
    id: f64,
    recv: Receiver<T>,
    _closure: Closure<dyn 'static + FnMut()>,
    _phtm: PhantomData<Closure<dyn 'a + FnOnce() -> T>>,
}

impl<'a, T> Timeout<'a, T> {
    #[inline]
    pub fn new<F: 'a + FnOnce() -> T>(f: F, timeout: Duration) -> Self {
        Self::new_with_millis(f, timeout2millis(timeout))
    }

    pub(crate) fn new_with_millis<F: 'a + FnOnce() -> T>(f: F, millis: i32) -> Self {
        let (send, recv) = channel::<T>();
        let mut f = Some(move || send.send(f()));

        let closure = Box::new(move || {
            let f = f.take().expect_throw("FnOnce called multiple times");
            f();
        });

        let closure = unsafe {
            core::mem::transmute::<Box<dyn 'a + FnMut()>, Box<dyn 'static + FnMut()>>(closure)
        };

        let closure = Closure::wrap(closure);
        let id = set_timeout(closure.as_ref(), millis);

        return Self {
            id,
            recv,
            _closure: closure,
            _phtm: PhantomData,
        };
    }
}

impl<'a, Fut: Future> Timeout<'a, Fut> {
    #[inline]
    pub fn new_async(fut: Fut, timeout: Duration) -> futures::future::Flatten<Self> {
        Self::new(move || fut, timeout).flatten()
    }
}

impl<'a, T> Timeout<'a, T> {
    #[inline]
    pub unsafe fn id(&self) -> f64 {
        self.id
    }

    #[inline]
    pub unsafe fn into_raw_parts(self) -> (f64, Receiver<T>, Closure<dyn 'static + FnMut()>) {
        unsafe {
            let this = ManuallyDrop::new(self);
            (
                this.id,
                core::ptr::read(&this.recv),
                core::ptr::read(&this._closure),
            )
        }
    }

    #[inline]
    pub unsafe fn from_raw_parts(
        id: f64,
        recv: Receiver<T>,
        closure: Closure<dyn 'static + FnMut()>,
    ) -> Self {
        return Self {
            id,
            recv,
            _closure: closure,
            _phtm: PhantomData,
        };
    }
}

impl<T> Timeout<'static, T> {
    #[inline]
    pub fn leak(self) -> (f64, Receiver<T>) {
        unsafe {
            let (id, recv, _) = self.into_raw_parts();
            (id, recv)
        }
    }
}

impl<'a, T> Future for Timeout<'a, T> {
    type Output = T;

    #[inline]
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.recv).poll_unchecked(cx)
    }
}

impl<T> Drop for Timeout<'_, T> {
    #[inline]
    fn drop(&mut self) {
        clear_timeout(self.id)
    }
}
