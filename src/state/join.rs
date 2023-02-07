use super::Subscriber;
use std::{cell::UnsafeCell, ptr::NonNull};

pub struct Join2<S: ?Sized, T: ?Sized, AF, BF> {
    inner: S,
    a: JoinCb<S, AF>,
    b: JoinCb<S, BF>,
    t: T,
}

impl<S, T, A, B, AF, BF> Join2<S, T, AF, BF>
where
    A: ?Sized,
    B: ?Sized,
    AF: FnMut(&A, &S) -> T,
    BF: FnMut(&B, &S) -> T,
{
    pub fn new (a: AF, b: BF) -> Self {
        let mut this = 0;
    }
}

/// A join's callback
struct JoinCb<S: ?Sized, F: ?Sized> {
    inner: NonNull<S>,
    f: UnsafeCell<F>,
}

impl<S: ?Sized, F> JoinCb<S, F> {
    #[inline]
    pub fn new (f: F) -> Self {
        Self { inner: NonNull::dangling(), f }
    }
}

impl<S, I, O, F> Subscriber<I> for JoinCb<S, F>
where
    I: ?Sized,
    S: ?Sized,
    F: ?Sized + FnMut(&I, &S) -> O,
{
    #[inline]
    fn update(&self, v: &I) {
        unsafe { (&mut *self.f.get())(v, self.inner.as_ref()) }
    }
}