use super::timeout2millis;
use crate::channel::oneshoot::{channel, Receiver};
use futures::Future;
use wasm_bindgen_futures::spawn_local;
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
    _phtm: PhantomData<(
        Closure<dyn 'a + FnOnce() -> T>,
        Closure<dyn 'a + Future<Output = T>>,
    )>,
}

impl<'a, T> Timeout<'a, T> {
    pub fn new<F: 'a + FnOnce() -> T> (f: F, timeout: Duration) -> Self {
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
        let id = set_timeout(closure.as_ref(), timeout2millis(timeout));

        return Self {
            id,
            recv,
            _closure: closure,
            _phtm: PhantomData,
        };
    }

    pub fn new_async<Fut: 'a + Future<Output = T>>(fut: Fut, timeout: Duration) -> Self where T: 'static {
        let (send, recv) = channel::<T>();
        let fut = Box::pin(async move { send.send(fut.await) });
        let mut fut = Some(unsafe {
            core::mem::transmute::<Pin<Box<dyn 'a + Future<Output = ()>>>, Pin<Box<dyn 'static + Future<Output = ()>>>>(fut)
        });

        let closure = Box::new(move || {
            let fut = fut.take().expect_throw("Future called multiple times");
            spawn_local(fut);
        });

        let closure = unsafe {
            core::mem::transmute::<Box<dyn 'a + FnMut()>, Box<dyn 'static + FnMut()>>(closure)
        };

        let closure = Closure::wrap(closure);
        let id = set_timeout(closure.as_ref(), timeout2millis(timeout));

        return Self {
            id,
            recv,
            _closure: closure,
            _phtm: PhantomData,
        };
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
