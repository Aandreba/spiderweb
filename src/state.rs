//flat_mod! { join }

use std::{
    cell::UnsafeCell,
    fmt::Display,
    rc::{self, Rc}, ops::Deref,
};

enum Strong<'a, T: ?Sized> {
    Callback(Box<dyn 'a + FnMut(&T)>),
    Listener(Rc<dyn 'a + Listener<T>>),
}

enum Weak<'a, T: ?Sized> {
    Callback(Box<dyn 'a + FnMut(&T) -> bool>),
    Listener(rc::Weak<dyn 'a + Listener<T>>),
}

/// A state cell that cannot be written into
#[repr(transparent)]
pub struct ReadOnlyState<'a, T: ?Sized> (pub(crate) StateCell<'a, T>);

pub struct StateCell<'a, T: ?Sized> {
    strong: UnsafeCell<Vec<Strong<'a, T>>>,
    weak: UnsafeCell<Vec<Weak<'a, T>>>,
    inner: UnsafeCell<T>,
}

pub trait Listener<T: ?Sized> {
    fn receive(&self, v: &T);
}

impl<'a, T: ?Sized> StateCell<'a, T> {
    #[inline]
    pub fn new(v: T) -> Self
    where
        T: Sized,
    {
        Self {
            inner: UnsafeCell::new(v),
            strong: UnsafeCell::default(),
            weak: UnsafeCell::default(),
        }
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
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        unsafe {
            f(&mut *self.inner.get());
            self.notify()
        }
    }

    unsafe fn notify(&self) {
        // Notify strongly-referenced subscribers
        // (we don't have to check if we drop them, we never will manually)
        for sub in (&mut *self.strong.get()).iter_mut() {
            sub.receive(&*self.inner.get())
        }

        let subs = &mut *self.weak.get();
        let mut i = 0;
        while i < subs.len() {
            if subs.get_unchecked_mut(i).receive(&*self.inner.get()) {
                i += 1
            } else {
                let _ = subs.swap_remove(i);
            }
        }
    }
}

impl<'a, T: ?Sized> ReadOnlyState<'a, T> {
    #[inline]
    pub fn new(v: T) -> Self
    where
        T: Sized,
    {
        Self(StateCell::new(v))
    }

    #[inline]
    pub fn with<U, F: FnOnce(&T) -> U>(&self, f: F) -> U {
        unsafe { f(&*self.0.inner.get()) }
    }

    #[inline]
    pub fn register<F: 'a + FnMut(&T)>(&self, f: F) {
        self.register_boxed(Box::new(f))
    }

    #[inline]
    pub fn register_weak<F: 'a + FnMut(&T) -> bool>(&self, f: F) {
        self.register_weak_boxed(Box::new(f))
    }

    #[inline]
    pub fn register_boxed(&self, f: Box<dyn 'a + FnMut(&T)>) {
        unsafe { &mut *self.0.strong.get() }.push(Strong::Callback(f));
    }

    #[inline]
    pub fn register_weak_boxed(&self, f: Box<dyn 'a + FnMut(&T) -> bool>) {
        unsafe { &mut *self.0.weak.get() }.push(Weak::Callback(f));
    }

    #[inline]
    pub fn bind(&self, sub: Rc<dyn 'a + Listener<T>>) {
        unsafe { &mut *self.0.strong.get() }.push(Strong::Listener(sub));
    }

    #[inline]
    pub fn bind_weak(&self, weak: rc::Weak<dyn 'a + Listener<T>>) {
        unsafe { &mut *self.0.weak.get() }.push(Weak::Listener(weak));
    }
}

impl<'a, T> From<StateCell<'a, T>> for ReadOnlyState<'a, T> {
    #[inline]
    fn from(value: StateCell<'a, T>) -> Self {
        Self(value)
    }
}

impl<'a, T: ?Sized> Deref for StateCell<'a, T> {
    type Target = ReadOnlyState<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*(self as *const Self as *const ReadOnlyState<'a, T>)
        }
    }
}

impl<T: ?Sized + Display> Display for StateCell<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { T::fmt(&*self.inner.get(), f) }
    }
}

impl<V: ?Sized, T: ?Sized + Listener<V>> Listener<V> for &T {
    #[inline]
    fn receive(&self, v: &V) {
        T::receive(*self, v)
    }
}

impl<V: ?Sized, T: ?Sized + Listener<V>> Listener<V> for Box<T> {
    #[inline]
    fn receive(&self, v: &V) {
        T::receive(self, v)
    }
}

impl<V: ?Sized, T: ?Sized + Listener<V>> Listener<V> for Rc<T> {
    #[inline]
    fn receive(&self, v: &V) {
        T::receive(self, v)
    }
}

impl<'a, T: ?Sized> Strong<'a, T> {
    #[inline]
    pub fn receive(&mut self, v: &T) {
        match self {
            Self::Callback(f) => f(v),
            Self::Listener(l) => l.receive(v),
        }
    }
}

impl<'a, T: ?Sized> Weak<'a, T> {
    #[inline]
    pub fn receive(&mut self, v: &T) -> bool {
        match self {
            Self::Callback(f) => f(v),
            Self::Listener(l) => match l.upgrade() {
                Some(l) => {
                    l.receive(v);
                    true
                }
                None => false,
            },
        }
    }
}
