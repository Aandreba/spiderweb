use futures::{Future, FutureExt};
use crate::noop;
use super::Timeout;
use std::{task::Poll, time::Duration};

/// Puts the current task to sleep for at least the specified amount of time.
///
/// The task may sleep longer than the duration specified due to scheduling
/// specifics or runtime-dependent functionality. It will never sleep less.
///
/// Unlinke [`Interval`] and [`Timeout`], this method will not saturate its duration if it exceedes [`MAX_DURATION`].
/// Instead, [`sleep`] will concatenate [`Timeout`]s until the desired duration is reached.
/// 
/// [`Interval`]: super::Interval
/// [`Timeout`]: super::Timeout
/// [`MAX_DURATION`]: super::MAX_DURATION
#[inline]
pub fn sleep(dur: Duration) -> Sleep {
    const LIMIT: u128 = i32::MAX as u128;

    let millis = dur.as_millis();
    #[cfg(feature = "nightly")]
    let map: TimeoutSet = TimeoutSet;
    #[cfg(not(feature = "nightly"))]
    let map: TimeoutSet = Box::new(|_| Timeout::new_with_millis(noop, i32::MAX));

    return match millis / LIMIT {
        0 => Sleep {
            iter: (0..0).map(map),
            current: Timeout::new_with_millis(noop, millis as i32)
        },

        div => Sleep {
            iter: (0..div).map(map),
            current: Timeout::new_with_millis(noop, (div % LIMIT) as i32)
        }
    }
}

#[cfg(not(feature = "nightly"))]
type TimeoutSet = Box<dyn 'static + Send + Sync + FnMut(u128) -> Timeout<'static, ()>>;

#[cfg(feature = "nightly")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TimeoutSet;

#[cfg(feature = "nightly")]
impl FnMut<(u128,)> for TimeoutSet {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, _args: (u128,)) -> Self::Output {
        Timeout::new_with_millis(noop, i32::MAX)
    }
}

#[cfg(feature = "nightly")]
impl FnOnce<(u128,)> for TimeoutSet {
    type Output = Timeout<'static, ()>;

    #[inline]
    extern "rust-call" fn call_once(self, _args: (u128,)) -> Self::Output {
        Timeout::new_with_millis(noop, i32::MAX)
    }
}

/// Future for [`sleep`]
pub struct Sleep {
    iter: std::iter::Map<std::ops::Range<u128>, TimeoutSet>,
    current: Timeout<'static, ()>,
}

impl Future for Sleep {
    type Output = ();

    #[inline]
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            match self.current.poll_unpin(cx) {
                Poll::Ready(_) => match self.iter.next() {
                    Some(x) => self.current = x,
                    None => return Poll::Ready(()),
                },
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}