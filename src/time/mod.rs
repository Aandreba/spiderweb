use std::time::Duration;
flat_mod! { interval, timeout, sleep }

/// Maximum ammount of time that can be passed to [`Interval`] or [`Timeout`].
///
/// If a grater duration than this one is passed to any of this types, their durations will
/// saturate to this value.
pub const MAX_DURATION: Duration = Duration::from_millis(i32::MAX as u64);

#[inline]
pub(super) fn timeout2millis(dur: Duration) -> i32 {
    match i32::try_from(dur.as_millis()) {
        Ok(x) => x,
        Err(_) => i32::MAX,
    }
}
