use futures::{future::FusedFuture, Future};
use std::{
    cell::{Cell, UnsafeCell},
    ops::{Deref, DerefMut},
    rc::Rc,
    task::{Poll, Waker},
};

pub type MutexLockFutureRef<'a, T> = MutexLockFuture<T, &'a Mutex<T>>;
pub type MutexLockFutureShared<T> = MutexLockFuture<T, Rc<Mutex<T>>>;

pub type MutexGuardRef<'a, T> = MutexGuard<T, &'a Mutex<T>>;
pub type MutexGuardShared<T> = MutexGuard<T, Rc<Mutex<T>>>;

#[derive(Debug, Clone)]
enum WakerCell {
    Uninit,
    Init(Waker),
}

/// A mutual exclusion primitive useful for protecting shared data.
///
/// This mutex will block tasks waiting for the lock to become available. The
/// mutex can also be statically initialized or created via a `new`
/// constructor. Each mutex has a type parameter which represents the data that
/// it is protecting. The data can only be accessed through the RAII guards
/// returned from `lock` and `try_lock`, which guarantees that the data is only
/// ever accessed when the mutex is locked.
#[derive(Debug)]
pub struct Mutex<T: ?Sized> {
    locked: Cell<bool>,
    wakers: UnsafeCell<Vec<WakerCell>>,
    empty_wakers: UnsafeCell<Vec<usize>>, // todo? switch to vecdeque
    inner: UnsafeCell<T>,
}

/// Future for [`lock_by_deref`](Mutex::lock_by_deref)
#[derive(Debug)]
pub struct MutexLockFuture<T: ?Sized, M: Deref<Target = Mutex<T>>> {
    inner: Option<M>,
    key: usize,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct MutexGuard<T: ?Sized, M: Deref<Target = Mutex<T>>>(M);

impl<T: ?Sized> Mutex<T> {
    #[inline]
    pub const fn new(t: T) -> Self
    where
        T: Sized,
    {
        return Self {
            locked: Cell::new(false),
            wakers: UnsafeCell::new(Vec::new()),
            empty_wakers: UnsafeCell::new(Vec::new()),
            inner: UnsafeCell::new(t),
        };
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    #[inline]
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.inner.into_inner()
    }

    #[inline]
    pub fn try_lock_by_deref<D: Deref<Target = Self>>(this: D) -> Result<MutexGuard<T, D>, D> {
        return match this.locked.get() {
            false => {
                this.locked.set(true);
                Ok(MutexGuard(this))
            }
            _ => Err(this),
        };
    }

    #[inline]
    pub fn lock_by_deref<D: Unpin + Deref<Target = Self>>(this: D) -> MutexLockFuture<T, D> {
        unsafe {
            let wakers = &mut *this.wakers.get();
            let empty_wakers = &mut *this.empty_wakers.get();

            let key = match empty_wakers.len() {
                0 => {
                    let i = wakers.len();
                    wakers.push(WakerCell::Uninit);
                    i
                }
                _ => empty_wakers.swap_remove(0),
            };

            return MutexLockFuture {
                inner: Some(this),
                key,
            };
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    #[inline]
    pub fn try_lock(&self) -> Option<MutexGuardRef<'_, T>> {
        Self::try_lock_by_deref(self).ok()
    }

    #[inline]
    pub fn lock(&self) -> MutexLockFutureRef<'_, T> {
        Self::lock_by_deref(self)
    }
}

impl<T: ?Sized> Mutex<T> {
    #[inline]
    pub fn try_lock_shared(self: Rc<Self>) -> Result<MutexGuardShared<T>, Rc<Self>> {
        Self::try_lock_by_deref(self)
    }

    #[inline]
    pub fn lock_shared(self: Rc<Self>) -> MutexLockFutureShared<T> {
        Self::lock_by_deref(self)
    }
}

impl WakerCell {
    #[inline]
    pub fn register(&mut self, waker: &Waker) {
        match self {
            Self::Uninit => *self = Self::Init(waker.clone()),
            Self::Init(x) if !x.will_wake(waker) => *x = waker.clone(),
            Self::Init(_) => {}
        }
    }

    #[inline]
    pub fn wake(&mut self) -> bool {
        match core::mem::replace(self, Self::Uninit) {
            Self::Init(waker) => {
                waker.wake();
                true
            }
            Self::Uninit => false,
        }
    }
}

impl<T: ?Sized, M: Deref<Target = Mutex<T>>> Deref for MutexGuard<T, M> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.inner.get() }
    }
}

impl<T: ?Sized, M: Deref<Target = Mutex<T>>> DerefMut for MutexGuard<T, M> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.inner.get() }
    }
}

impl<T: ?Sized, M: Deref<Target = Mutex<T>>> Drop for MutexGuard<T, M> {
    #[inline]
    fn drop(&mut self) {
        self.0.locked.set(false);
        for waker in unsafe { &mut *self.0.wakers.get() }.iter_mut() {
            if waker.wake() {
                break;
            }
        }
    }
}

impl<T: ?Sized, M: Unpin + Deref<Target = Mutex<T>>> Future for MutexLockFuture<T, M> {
    type Output = MutexGuard<T, M>;

    #[inline]
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(ref inner) = self.inner {
            unsafe {
                match inner.locked.get() {
                    false => {
                        inner.locked.set(true);
                        uninit_waker(inner, self.key);
                        return Poll::Ready(MutexGuard(self.inner.take().unwrap_unchecked()));
                    }
                    _ => {
                        (&mut *inner.wakers.get())
                            .get_unchecked_mut(self.key)
                            .register(cx.waker());
                        return Poll::Pending;
                    }
                }
            }
        }

        crate::eprintln!("This future has already completed");
        return Poll::Pending;
    }
}

impl<T: ?Sized, M: Unpin + Deref<Target = Mutex<T>>> FusedFuture for MutexLockFuture<T, M> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_none()
    }
}

impl<T: ?Sized, M: Deref<Target = Mutex<T>>> Drop for MutexLockFuture<T, M> {
    #[inline]
    fn drop(&mut self) {
        if let Some(ref inner) = self.inner {
            unsafe { uninit_waker(inner, self.key) }
        }
    }
}

#[inline]
unsafe fn uninit_waker<T: ?Sized, M: Deref<Target = Mutex<T>>>(inner: &M, key: usize) {
    *(&mut *inner.wakers.get()).get_unchecked_mut(key) = WakerCell::Uninit;
    (&mut *inner.empty_wakers.get()).push(key);
}
