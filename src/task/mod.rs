use crate::{time::Timeout, noop};

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
    Timeout::new_with_millis(noop, 0i32).await
}
