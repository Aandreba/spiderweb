use crate::{
    channel::oneshoot::{channel, Receiver, Sender},
    time::Timeout,
};
use futures::{Future, FutureExt};
use std::{marker::PhantomData, pin::Pin, task::Poll, time::Duration};
use wasm_bindgen_futures::spawn_local;

pin_project_lite::pin_project! {
    pub struct Task<'a, T: 'static> {
        abort: Sender<()>,
        #[pin]
        recv: Receiver<T>,
        _phtm: PhantomData<dyn 'a + Future<Output = T>>,
    }
}

impl<'a, T> Task<'a, T> {
    #[inline]
    pub fn new<Fut: 'a + Future<Output = T>>(fut: Fut) -> Self {
        Self::new_boxed(Box::pin(fut))
    }

    #[inline]
    pub fn new_boxed(fut: Pin<Box<dyn 'a + Future<Output = T>>>) -> Self {
        struct Inner<T> {
            inner: Pin<Box<dyn 'static + Future<Output = T>>>,
            abort: Receiver<()>,
            send: Option<Sender<T>>,
        }

        impl<T> Future for Inner<T> {
            type Output = ();

            #[inline]
            fn poll(
                mut self: Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                if self.abort.poll_unpin(cx).is_ready() {
                    return Poll::Ready(());
                }

                if let Poll::Ready(x) = self.inner.poll_unpin(cx) {
                    if let Some(send) = self.send.take() {
                        send.send(x);
                    }
                    return Poll::Ready(());
                }

                return Poll::Pending;
            }
        }

        let (send, recv) = channel::<T>();
        let (abort_send, abort_recv) = channel::<()>();
        spawn_local(Inner::<T> {
            inner: unsafe { core::mem::transmute(fut) },
            abort: abort_recv,
            send: Some(send),
        });

        return Self {
            abort: abort_send,
            recv,
            _phtm: PhantomData,
        }
    }
}

impl<'a, T> Future for Task<'a, T> {
    type Output = T;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.project().recv.poll_unchecked(cx)
    }
}

/// Cooperatively gives up a timeslice to the JavaScript scheduler.
///
/// This calls the underlying JavaScript scheduler's yield primitive, signaling
/// that the calling task is willing to give up its remaining timeslice
/// so that JavaScript may schedule other tasks on the CPU.
///
/// A drawback of yielding in a loop is that if JavaScript does not have any
/// other ready tasks to run on the current CPU, the task will effectively
/// busy-wait, which wastes CPU time and energy.
///
/// `yield_now` should thus be used only rarely, mostly in situations where
/// repeated polling is required because there is no other suitable way to
/// learn when an event of interest has occurred.
///
/// # Examples
///
/// ```
/// use safeweb::task;
///
/// task::yield_now();
/// ```
///
/// [`channel`]: crate::channel
#[inline]
pub async fn yield_now() {
    Timeout::new(<()>::default, Duration::ZERO).await
}
