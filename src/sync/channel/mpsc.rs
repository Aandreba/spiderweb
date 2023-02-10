use futures::Stream;
use std::{
    cell::{Cell, UnsafeCell},
    collections::VecDeque,
    fmt::Debug,
    rc::{Rc, Weak},
    task::Waker,
};

/// Creates a new MPSC channel
#[inline]
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: UnsafeCell::new(VecDeque::new()),
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
    queue: UnsafeCell<VecDeque<T>>,
    waker: Cell<Option<Waker>>,
}

/// Sender of [`channel`]
#[derive(Debug)]
pub struct Sender<T> {
    inner: Weak<Inner<T>>,
}

/// Receiver of [`channel`]
#[derive(Debug)]
pub struct Receiver<T> {
    inner: Rc<Inner<T>>,
}

impl<T> Sender<T> {
    #[inline]
    pub fn send(&self, v: T) {
        let _ = self.try_send(v);
    }

    #[inline]
    pub fn try_send(&self, v: T) -> Result<(), T> {
        if let Some(inner) = self.inner.upgrade() {
            unsafe { &mut *inner.queue.get() }.push_back(v);
            if let Some(waker) = inner.waker.take() {
                waker.wake()
            }
            return Ok(());
        }

        return Err(v);
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    #[inline]
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match unsafe { &mut *self.inner.queue.get() }.pop_front() {
            Some(x) => std::task::Poll::Ready(Some(x)),
            None if Rc::weak_count(&self.inner) == 0 => std::task::Poll::Ready(None),
            _ => {
                self.inner.waker.set(Some(cx.waker().clone()));
                std::task::Poll::Pending
            }
        }
    }
}

impl<T> Clone for Sender<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Debug> Debug for Inner<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner")
            .field("queue", &self.queue)
            .finish_non_exhaustive()
    }
}
