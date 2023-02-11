use std::{time::Duration, ops::{Add, AddAssign, Sub, SubAssign}};
use crate::dom::now;

/// A measurement of a monotonically nondecreasing clock.
/// Opaque and useful only with [`Duration`].
///
/// Instants are always guaranteed, barring [platform bugs], to be no less than any previously
/// measured instant when created, and are often useful for tasks such as measuring
/// benchmarks or timing how long an operation takes.
///
/// Note, however, that instants are **not** guaranteed to be **steady**. In other
/// words, each tick of the underlying clock might not be the same length (e.g.
/// some seconds may be longer than others). An instant may jump forwards or
/// experience time dilation (slow down or speed up), but it will never go
/// backwards.
///
/// Instants are opaque types that can only be compared to one another. There is
/// no method to get "the number of seconds" from an instant. Instead, it only
/// allows measuring the duration between two instants (or comparing two
/// instants).
///
/// The size of an `Instant` struct may vary depending on the target operating
/// system.
///
/// Example:
///
/// ```no_run
/// use spiderweb::time::{Duration, Instant};
/// use spiderweb::task::sleep;
///
/// async fn main() {
///    let now = Instant::now();
///
///    // we sleep for 2 seconds
///    sleep(Duration::new(2, 0)).await;
///    // it prints '2'
///    println!("{}", now.elapsed().as_secs());
/// }
/// ```
///
/// [platform bugs]: Instant#monotonicity
///
/// # OS-specific behaviors
///
/// An `Instant` is a wrapper around system-specific types and it may behave
/// differently depending on the underlying operating system. For example,
/// the following snippet is fine on Linux but panics on macOS:
///
/// ```no_run
/// use spiderweb::time::{Instant, Duration};
///
/// let now = Instant::now();
/// let max_seconds = u64::MAX / 1_000_000_000;
/// let duration = Duration::new(max_seconds, 0);
/// println!("{:?}", now + duration);
/// ```
///
/// # Underlying System calls
///
/// The following system calls are [currently] being used by `now()` to find out
/// the current time:
///
/// |  Platform |               System call                                            |
/// |-----------|----------------------------------------------------------------------|
/// | SGX       | [`insecure_time` usercall]. More information on [timekeeping in SGX] |
/// | UNIX      | [clock_gettime (Monotonic Clock)]                                    |
/// | Darwin    | [mach_absolute_time]                                                 |
/// | VXWorks   | [clock_gettime (Monotonic Clock)]                                    |
/// | SOLID     | `get_tim`                                                            |
/// | WASI      | [__wasi_clock_time_get (Monotonic Clock)]                            |
/// | WASM (JS) | [`performance.now`]                                                  |
/// | Windows   | [QueryPerformanceCounter]                                            |
///
/// [currently]: std::io#platform-specific-behavior
/// [QueryPerformanceCounter]: https://docs.microsoft.com/en-us/windows/win32/api/profileapi/nf-profileapi-queryperformancecounter
/// [`insecure_time` usercall]: https://edp.fortanix.com/docs/api/fortanix_sgx_abi/struct.Usercalls.html#method.insecure_time
/// [timekeeping in SGX]: https://edp.fortanix.com/docs/concepts/rust-std/#codestdtimecode
/// [__wasi_clock_time_get (Monotonic Clock)]: https://github.com/WebAssembly/WASI/blob/master/phases/snapshot/docs.md#clock_time_get
/// [clock_gettime (Monotonic Clock)]: https://linux.die.net/man/3/clock_gettime
/// [mach_absolute_time]: https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/KernelProgramming/services/services.html
/// [`performance.now`]: https://developer.mozilla.org/en-US/docs/Web/API/Performance/now
/// 
/// **Disclaimer:** These system calls might change over time.
///
/// > Note: mathematical operations like [`add`] may panic if the underlying
/// > structure cannot represent the new point in time.
///
/// [`add`]: Instant::add
///
/// ## Monotonicity
///
/// On all platforms `Instant` will try to use an OS API that guarantees monotonic behavior
/// if available, which is the case for all [tier 1] platforms.
/// In practice such guarantees are – under rare circumstances – broken by hardware, virtualization
/// or operating system bugs. To work around these bugs and platforms not offering monotonic clocks
/// [`duration_since`], [`elapsed`] and [`sub`] saturate to zero. In older Rust versions this
/// lead to a panic instead. [`checked_duration_since`] can be used to detect and handle situations
/// where monotonicity is violated, or `Instant`s are subtracted in the wrong order.
///
/// This workaround obscures programming errors where earlier and later instants are accidentally
/// swapped. For this reason future rust versions may reintroduce panics.
///
/// [tier 1]: https://doc.rust-lang.org/rustc/platform-support.html
/// [`duration_since`]: Instant::duration_since
/// [`elapsed`]: Instant::elapsed
/// [`sub`]: Instant::sub
/// [`checked_duration_since`]: Instant::checked_duration_since
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant (pub(crate) Duration);

impl Instant {
    /// Returns an instant corresponding to "now".
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderweb::time::Instant;
    ///
    /// let now = Instant::now();
    /// ```
    #[inline]
    #[must_use]
    pub fn now () -> Self {
        Self(Duration::from_secs_f64(now() / 1000.))
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    ///
    /// # Panics
    ///
    /// Previous rust versions panicked when `earlier` was later than `self`. Currently this
    /// method saturates. Future versions may reintroduce the panic in some circumstances.
    /// See [Monotonicity].
    ///
    /// [Monotonicity]: Instant#monotonicity
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderweb::time::{Duration, Instant};
    /// use spiderweb::task::sleep;
    ///
    /// let now = Instant::now();
    /// sleep(Duration::new(1, 0)).await;
    /// let new_now = Instant::now();
    /// println!("{:?}", new_now.duration_since(now));
    /// println!("{:?}", now.duration_since(new_now)); // 0ns
    /// ```
    #[inline]
    #[must_use]
    pub fn duration_since (&self, earlier: Instant) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or None if that instant is later than this one.
    ///
    /// Due to [monotonicity bugs], even under correct logical ordering of the passed `Instant`s,
    /// this method can return `None`.
    ///
    /// [monotonicity bugs]: Instant#monotonicity
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderweb::time::{Duration, Instant};
    /// use spiderweb::task::sleep;
    ///
    /// let now = Instant::now();
    /// sleep(Duration::new(1, 0)).await;
    /// let new_now = Instant::now();
    /// println!("{:?}", new_now.checked_duration_since(now));
    /// println!("{:?}", now.checked_duration_since(new_now)); // None
    /// ```
    #[inline]
    #[must_use]
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        self.0.checked_sub(earlier.0)
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderweb::time::{Duration, Instant};
    /// use spiderweb::task::sleep;
    ///
    /// let now = Instant::now();
    /// sleep(Duration::new(1, 0));
    /// let new_now = Instant::now();
    /// println!("{:?}", new_now.saturating_duration_since(now));
    /// println!("{:?}", now.saturating_duration_since(new_now)); // 0ns
    /// ```
    #[inline]
    #[must_use]
    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    /// Returns the amount of time elapsed since this instant was created.
    ///
    /// # Panics
    ///
    /// Previous rust versions panicked when the current time was earlier than self. Currently this
    /// method returns a Duration of zero in that case. Future versions may reintroduce the panic.
    /// See [Monotonicity].
    ///
    /// [Monotonicity]: Instant#monotonicity
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderweb::time::{Duration, Instant};
    /// use spiderweb::task::sleep;
    ///
    /// let instant = Instant::now();
    /// let three_secs = Duration::from_secs(3);
    /// sleep(three_secs).await;
    /// assert!(instant.elapsed() >= three_secs);
    /// ```
    #[inline]
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    #[inline]
    pub fn checked_add (&self, rhs: Duration) -> Option<Self> {
        self.0.checked_add(rhs).map(Self)
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    #[inline]
    pub fn checked_sub (&self, rhs: Duration) -> Option<Self> {
        self.0.checked_sub(rhs).map(Self)
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    #[inline]
    fn add(self, rhs: Duration) -> Self::Output {
        self.checked_add(rhs).expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for Instant {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    #[inline]
    fn sub(self, rhs: Duration) -> Self::Output {
        self.checked_sub(rhs).expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for Instant {
    #[inline]
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Instant) -> Self::Output {
        self.duration_since(rhs)
    }
}