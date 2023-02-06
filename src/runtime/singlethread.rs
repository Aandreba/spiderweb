use std::pin::Pin;
use futures::Future;
use wasm_bindgen::{prelude::{wasm_bindgen, Closure}, JsValue};
use crate::channel::oneshoot::{Receiver, channel};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = queueMicrotask)]
    fn queue_microtask (f: &JsValue);
}

#[inline]
pub fn spawn<T: 'static, Fut: 'static + Future<Output = T>> (fut: Fut) -> JoinHandle<T> {
    let (send, recv) = channel::<T>();
    let closure = Closure::new(move || {
        match fut.poll
    });

    queue_microtask(closure.as_ref());
    return JoinHandle {
        recv,
        _closure: closure
    }
}

#[inline]
pub fn spawn_blocking<T: 'static, F: 'static + FnOnce() -> T> (f: F) -> JoinHandle<T> {
    let (send, recv) = channel::<T>();
    let closure = Closure::once(move || send.send(f()));

    queue_microtask(closure.as_ref());
    return JoinHandle {
        recv,
        _closure: closure
    }
}

pub struct JoinHandle<T> {
    recv: Receiver<T>,
    _closure: Closure<dyn 'static + FnMut()> 
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    #[inline]
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.recv).poll_unchecked(cx)
    }
}