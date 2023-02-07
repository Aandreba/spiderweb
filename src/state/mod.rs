//flat_mod! { join }

use std::{
    cell::UnsafeCell,
    fmt::Display,
    rc::{Rc, Weak},
};

#[derive(Clone)]
enum StateSub<'a, T: ?Sized> {
    Strong(Rc<dyn 'a + Subscriber<T>>),
    Weak(Weak<dyn 'a + Subscriber<T>>),
}

pub struct State<'a, T: ?Sized> {
    subs: UnsafeCell<Vec<StateSub<'a, T>>>,
    inner: UnsafeCell<T>,
}

pub trait Subscriber<T: ?Sized> {
    fn update(&self, v: &T);
}

impl<'a, T: ?Sized> State<'a, T> {
    #[inline]
    pub fn new(v: T) -> Self
    where
        T: Sized,
    {
        Self {
            inner: UnsafeCell::new(v),
            subs: UnsafeCell::new(Vec::new()),
        }
    }

    #[inline]
    pub fn with<U, F: FnOnce(&T) -> U>(&self, f: F) -> U {
        unsafe { f(&*self.inner.get()) }
    }

    #[inline]
    pub fn set(&self, v: T)
    where
        T: Sized,
    {
        unsafe {
            *self.inner.get() = v;
            self.notify()
        }
    }

    #[inline]
    pub fn mutate<F: FnOnce(&mut T)>(&self, f: F) {
        unsafe {
            f(&mut *self.inner.get());
            self.notify()
        }
    }

    #[inline]
    unsafe fn notify(&self) {
        let subs = &mut *self.subs.get();

        let mut i = 0;
        while i < subs.len() {
            match subs.get_unchecked(i) {
                StateSub::Strong(sub) => {
                    sub.update(&*self.inner.get());
                    i += 1
                }

                StateSub::Weak(sub) => {
                    if let Some(sub) = sub.upgrade() {
                        sub.update(&*self.inner.get());
                        i += 1
                    } else {
                        let _ = subs.swap_remove(i);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn bind(&self, sub: Rc<dyn 'a + Subscriber<T>>) {
        unsafe { &mut *self.subs.get() }.push(StateSub::Strong(sub));
    }

    #[inline]
    pub fn bind_weak(&self, weak: Weak<dyn 'a + Subscriber<T>>) {
        unsafe { &mut *self.subs.get() }.push(StateSub::Weak(weak));
    }
}

impl<T: ?Sized + Display> Display for State<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { T::fmt(&*self.inner.get(), f) }
    }
}

impl<V: ?Sized, T: ?Sized + Subscriber<V>> Subscriber<V> for &T {
    #[inline]
    fn update(&self, v: &V) {
        T::update(*self, v)
    }
}

impl<V: ?Sized, T: ?Sized + Subscriber<V>> Subscriber<V> for Box<T> {
    #[inline]
    fn update(&self, v: &V) {
        T::update(self, v)
    }
}

impl<V: ?Sized, T: ?Sized + Subscriber<V>> Subscriber<V> for Rc<T> {
    #[inline]
    fn update(&self, v: &V) {
        T::update(self, v)
    }
}
