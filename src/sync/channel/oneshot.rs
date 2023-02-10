use futures::Future;
use std::{
    cell::Cell,
    mem::ManuallyDrop,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

/// Creates a new oneshoot channel
#[inline]
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        value: Cell::new(None),
        waker: Cell::new(None),
    };

    let inner = Rc::new(inner);
    return (
        Sender {
            inner: Rc::downgrade(&inner),
        },
        Receiver { inner },
    );
}

struct Inner<T> {
    value: Cell<Option<T>>,
    waker: Cell<Option<Waker>>,
}

/// Sender of [`channel`]
pub struct Sender<T> {
    inner: Weak<Inner<T>>,
}

/// Receiver of [`channel`].
///
/// If the channel's [`Sender`] is dropped before it sends any value, this future
/// will return `None`.
pub struct Receiver<T> {
    inner: Rc<Inner<T>>,
}

impl<T> Sender<T> {
    #[inline]
    pub fn send(self, v: T) {
        let _ = self.try_send(v);
    }

    #[inline]
    pub fn try_send(self, v: T) -> Result<(), T> {
        let mut this = ManuallyDrop::new(self);
        if let Some(inner) = this.inner.upgrade() {
            inner.value.set(Some(v));
            if let Some(waker) = inner.waker.take() {
                waker.wake()
            }

            unsafe { core::ptr::drop_in_place(&mut this.inner) }
            return Ok(());
        }

        unsafe { core::ptr::drop_in_place(&mut this.inner) }
        return Err(v);
    }
}

impl<T> Receiver<T> {
    #[inline]
    pub fn is_available (&self) -> bool {
        unsafe { &*self.inner.value.as_ptr() }.is_some()
    }

    #[inline]
    pub fn poll_unchecked(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        match self.inner.value.take() {
            Some(x) => std::task::Poll::Ready(x),
            None => {
                self.inner.waker.set(Some(cx.waker().clone()));
                std::task::Poll::Pending
            }
        }
    }
}

impl<T> Future for Receiver<T> {
    type Output = Option<T>;

    #[inline]
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.inner.value.take() {
            Some(x) => std::task::Poll::Ready(Some(x)),
            None if Rc::weak_count(&self.inner) == 0 => std::task::Poll::Ready(None),
            _ => {
                self.inner.waker.set(Some(cx.waker().clone()));
                std::task::Poll::Pending
            }
        }
    }
}

impl<T> Drop for Sender<T> {
    #[inline]
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            if let Some(waker) = inner.waker.take() {
                waker.wake()
            }
        }
    }
}
