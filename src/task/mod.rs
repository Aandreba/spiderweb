use std::task::Poll;
use futures::{Future, FutureExt};
use crate::{noop, time::Timeout};

flat_mod! { sleep }

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
/// task::yield_now().await;
/// ```
///
/// [`channel`]: crate::channel
#[inline]
pub fn yield_now() -> YieldNow {
    YieldNow(Some(Timeout::new_with_millis(noop, 0i32)))
}

/// Future for [`yield_now`]
#[repr(transparent)]
pub struct YieldNow (Option<Timeout<'static, ()>>);

impl Future for YieldNow {
    type Output = ();

    #[inline]
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if let Some(ref mut inner) = self.0 {
            if inner.poll_unpin(cx).is_ready() {
                self.0 = None;
                return Poll::Ready(())
            } else {
                return Poll::Pending
            }
        } 

        return Poll::Ready(())
    }
}