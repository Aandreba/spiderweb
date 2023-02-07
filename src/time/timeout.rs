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
    fn set_timeout(handler: &JsValue, millis: i32) -> f64;
    #[wasm_bindgen(js_name = clearTimeout)]
    fn clear_timeout(id: f64);
}

/// Handler of a JavaScript timeout.
/// 
/// After the specified time delay, the handle's callback will be called, and it's return value will become available
/// to the future.
/// 
/// When dropped, the timeout will be automatically cleared, it's underlying channel closed.
pub struct Timeout<'a, T: 'a> {
    id: f64,
    recv: Receiver<T>,
    _closure: Closure<dyn 'static + FnMut()>,
    _phtm: PhantomData<Closure<dyn 'a + FnOnce() -> T>>,
}

impl<'a, T> Timeout<'a, T> {
    /// Creates a new timeout that executes `f` after the specified time delay of `timeout`. 
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
    /// Creates a timeout that resolves to a [`Future`], and then it (the `Timeout`) is immediately flattened,
    /// returning a [`Future`] that resolves to the timeout's future output
    #[inline]
    pub fn new_async(fut: Fut, timeout: Duration) -> futures::future::Flatten<Self> {
        Self::new(move || fut, timeout).flatten()
    }
}

impl<'a, T> Timeout<'a, T> {
    /// Returns the id of the timeout.
    /// 
    /// # Safety
    /// This handler must be forgoten, or this id must not be used to clear the timeout manually.
    /// Both things ocurring is considered undefined behavior.
    #[inline]
    pub unsafe fn id(&self) -> f64 {
        self.id
    }

    /// Converts the handler into its raw components.
    /// 
    /// After calling this method, the caller is responsable for dropping the closure and clearing the
    /// timeout.
    #[inline]
    pub fn into_raw_parts(self) -> (f64, Receiver<T>, Closure<dyn 'static + FnMut()>) {
        unsafe {
            let this = ManuallyDrop::new(self);
            (
                this.id,
                core::ptr::read(&this.recv),
                core::ptr::read(&this._closure),
            )
        }
    }

    /// Creates a handle from its raw components.
    /// 
    /// # Safety
    /// Calling this function with elements that haven't originated from [`into_raw_parts`]
    /// is likely to be undefined behavior.
    /// 
    /// [`into_raw_parts`]: Timeout::into_raw_parts
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
    /// Consumes and leaks the timeout, returning it's id and receiver. 
    #[inline]
    pub fn leak(self) -> (f64, Receiver<T>) {
        let (id, recv, _) = self.into_raw_parts();
        (id, recv)
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
