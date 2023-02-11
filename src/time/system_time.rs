use std::{time::Duration, ops::{AddAssign, Sub, SubAssign, Add}};
use wasm_bindgen::JsValue;
#[allow(unused)]
use super::Instant;

/// An anchor in time which can be used to create new `SystemTime` instances or
/// learn about where in time a `SystemTime` lies.
///
/// This constant is defined to be "1970-01-01 00:00:00 UTC" on all systems with
/// respect to the system clock. Using `duration_since` on an existing
/// [`SystemTime`] instance can tell how far away from this point in time a
/// measurement lies, and using `UNIX_EPOCH + duration` can be used to create a
/// [`SystemTime`] instance to represent another fixed point in time.
///
/// # Examples
///
/// ```no_run
/// use spiderweb::time::{SystemTime, UNIX_EPOCH};
///
/// match SystemTime::now().duration_since(UNIX_EPOCH) {
///     Ok(n) => println!("1970-01-01 00:00:00 UTC was {} seconds ago!", n.as_secs()),
///     Err(_) => panic!("SystemTime before UNIX EPOCH!"),
/// }
/// ```
pub const UNIX_EPOCH: SystemTime = SystemTime(Duration::ZERO);

/// An error returned from the `duration_since` and `elapsed` methods on
/// `SystemTime`, used to learn how far in the opposite direction a system time
/// lies.
///
/// # Examples
///
/// ```no_run
/// use spiderweb::task::sleep;
/// use spiderweb::time::{Duration, SystemTime};
///
/// let sys_time = SystemTime::now();
/// sleep(Duration::from_secs(1)).await;
/// let new_sys_time = SystemTime::now();
/// match sys_time.duration_since(new_sys_time) {
///     Ok(_) => {}
///     Err(e) => println!("SystemTimeError difference: {:?}", e.duration()),
/// }
/// ```
#[derive(Clone, Debug)]
pub struct SystemTimeError(Duration);

/// A measurement of the system clock, useful for talking to
/// external entities like the file system or other processes.
///
/// Distinct from the [`Instant`] type, this time measurement **is not
/// monotonic**. This means that you can save a file to the file system, then
/// save another file to the file system, **and the second file has a
/// `SystemTime` measurement earlier than the first**. In other words, an
/// operation that happens after another operation in real time may have an
/// earlier `SystemTime`!
///
/// Consequently, comparing two `SystemTime` instances to learn about the
/// duration between them returns a [`Result`] instead of an infallible [`Duration`]
/// to indicate that this sort of time drift may happen and needs to be handled.
///
/// Although a `SystemTime` cannot be directly inspected, the [`UNIX_EPOCH`]
/// constant is provided in this module as an anchor in time to learn
/// information about a `SystemTime`. By calculating the duration from this
/// fixed point in time, a `SystemTime` can be converted to a human-readable time,
/// or perhaps some other string representation.
///
/// The size of a `SystemTime` struct may vary depending on the target operating
/// system.
///
/// Example:
///
/// ```no_run
/// use std::time::{Duration, SystemTime};
/// use std::thread::sleep;
///
/// fn main() {
///    let now = SystemTime::now();
///
///    // we sleep for 2 seconds
///    sleep(Duration::new(2, 0));
///    match now.elapsed() {
///        Ok(elapsed) => {
///            // it prints '2'
///            println!("{}", elapsed.as_secs());
///        }
///        Err(e) => {
///            // an error occurred!
///            println!("Error: {e:?}");
///        }
///    }
/// }
/// ```
///
/// # Platform-specific behavior
///
/// The precision of `SystemTime` can depend on the underlying OS-specific time format.
/// For example, on Windows the time is represented in 100 nanosecond intervals whereas Linux
/// can represent nanosecond intervals.
///
/// The following system calls are [currently] being used by `now()` to find out
/// the current time:
///
/// |  Platform |               System call                                            |
/// |-----------|----------------------------------------------------------------------|
/// | SGX       | [`insecure_time` usercall]. More information on [timekeeping in SGX] |
/// | UNIX      | [clock_gettime (Realtime Clock)]                                     |
/// | Darwin    | [gettimeofday]                                                       |
/// | VXWorks   | [clock_gettime (Realtime Clock)]                                     |
/// | SOLID     | `SOLID_RTC_ReadTime`                                                 |
/// | WASI      | [__wasi_clock_time_get (Realtime Clock)]                             |
/// | WASM (JS) | [`Date.now`]                                                         |
/// | Windows   | [GetSystemTimePreciseAsFileTime] / [GetSystemTimeAsFileTime]         |
///
/// [currently]: std::io#platform-specific-behavior
/// [`insecure_time` usercall]: https://edp.fortanix.com/docs/api/fortanix_sgx_abi/struct.Usercalls.html#method.insecure_time
/// [timekeeping in SGX]: https://edp.fortanix.com/docs/concepts/rust-std/#codestdtimecode
/// [gettimeofday]: https://man7.org/linux/man-pages/man2/gettimeofday.2.html
/// [clock_gettime (Realtime Clock)]: https://linux.die.net/man/3/clock_gettime
/// [__wasi_clock_time_get (Realtime Clock)]: https://github.com/WebAssembly/WASI/blob/master/phases/snapshot/docs.md#clock_time_get
/// [GetSystemTimePreciseAsFileTime]: https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemtimepreciseasfiletime
/// [GetSystemTimeAsFileTime]: https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemtimeasfiletime
/// [`Date.now`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/now
///
/// **Disclaimer:** These system calls might change over time.
///
/// > Note: mathematical operations like [`add`] may panic if the underlying
/// > structure cannot represent the new point in time.
///
/// [`add`]: SystemTime::add
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime(pub(crate) Duration);

impl SystemTime {
    pub const UNIX_EPOCH: SystemTime = UNIX_EPOCH;

    /// Returns the system time corresponding to "now".
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderweb::time::SystemTime;
    ///
    /// let sys_time = SystemTime::now();
    /// ```
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        Self(Duration::from_secs_f64(js_sys::Date::now() / 1000.))
    }

    /// Returns the amount of time elapsed from an earlier point in time.
    ///
    /// This function may fail because measurements taken earlier are not
    /// guaranteed to always be before later measurements (due to anomalies such
    /// as the system clock being adjusted either forwards or backwards).
    /// [`Instant`] can be used to measure elapsed time without this risk of failure.
    ///
    /// If successful, <code>[Ok]\([Duration])</code> is returned where the duration represents
    /// the amount of time elapsed from the specified measurement to this one.
    ///
    /// Returns an [`Err`] if `earlier` is later than `self`, and the error
    /// contains how far from `self` the time is.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderweb::time::SystemTime;
    ///
    /// let sys_time = SystemTime::now();
    /// let new_sys_time = SystemTime::now();
    /// let difference = new_sys_time.duration_since(sys_time)
    ///     .expect("Clock may have gone backwards");
    /// println!("{difference:?}");
    /// ```
    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, SystemTimeError> {
        match self.0.checked_sub(earlier.0) {
            Some(x) => Ok(x),
            None => Err(SystemTimeError(earlier.0 - self.0)),
        }
    }

    /// Returns the difference between the clock time when this
    /// system time was created, and the current clock time.
    ///
    /// This function may fail as the underlying system clock is susceptible to
    /// drift and updates (e.g., the system clock could go backwards), so this
    /// function might not always succeed. If successful, <code>[Ok]\([Duration])</code> is
    /// returned where the duration represents the amount of time elapsed from
    /// this time measurement to the current time.
    ///
    /// To measure elapsed time reliably, use [`Instant`] instead.
    ///
    /// Returns an [`Err`] if `self` is later than the current system time, and
    /// the error contains how far from the current system time `self` is.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderweb::task::sleep;
    /// use spiderweb::time::{Duration, SystemTime};
    ///
    /// let sys_time = SystemTime::now();
    /// let one_sec = Duration::from_secs(1);
    /// sleep(one_sec).await;
    /// assert!(sys_time.elapsed().unwrap() >= one_sec);
    /// ```
    #[inline]
    pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        SystemTime::now().duration_since(*self)
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
    /// `SystemTime` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    #[inline]
    pub fn checked_add(&self, duration: Duration) -> Option<SystemTime> {
        self.0.checked_add(duration).map(Self)
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
    /// `SystemTime` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    #[inline]
    pub fn checked_sub(&self, duration: Duration) -> Option<SystemTime> {
        self.0.checked_sub(duration).map(Self)
    }
}

impl Add<Duration> for SystemTime {
    type Output = SystemTime;

    /// # Panics
    ///
    /// This function may panic if the resulting point in time cannot be represented by the
    /// underlying data structure. See [`SystemTime::checked_add`] for a version without panic.
    #[inline]
    fn add(self, dur: Duration) -> SystemTime {
        self.checked_add(dur).expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for SystemTime {
    #[inline]
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for SystemTime {
    type Output = SystemTime;

    #[inline]
    fn sub(self, dur: Duration) -> SystemTime {
        self.checked_sub(dur).expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for SystemTime {
    #[inline]
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl Into<js_sys::Date> for SystemTime {
    #[inline]
    fn into(self) -> js_sys::Date {
        js_sys::Date::new(&JsValue::from_f64(
            1000. * (self.0.as_secs() as f64) + (self.0.as_millis() as f64)
        ))
    }
}

impl From<js_sys::Date> for SystemTime {
    #[inline]
    fn from(value: js_sys::Date) -> Self {
        SystemTime(Duration::from_secs_f64(value.get_time() / 1000.))
    }
}
