use std::time::Duration;
flat_mod! { interval, timeout }

pub const MAX_DURATION: Duration = Duration::from_millis(i32::MAX as u64);

#[inline]
pub(super) fn timeout2millis (dur: Duration) -> i32 {
    match i32::try_from(dur.as_millis()) {
        Ok(x) => x,
        Err(_) => i32::MAX
    }
}