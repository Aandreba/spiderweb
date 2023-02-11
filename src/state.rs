//flat_mod! { join }

use std::{
    cell::UnsafeCell,
    fmt::Display,
    rc::{self, Rc}, ops::*,
};

enum Strong<'a, T: ?Sized> {
    Callback(Box<dyn 'a + FnMut(&T)>),
    Listener(Rc<dyn 'a + Listener<T>>),
}

enum Weak<'a, T: ?Sized> {
    Callback(Box<dyn 'a + FnMut(&T) -> bool>),
    Listener(rc::Weak<dyn 'a + Listener<T>>),
}

/// A state cell that cannot be written to
#[repr(transparent)]
pub struct ReadState<'a, T: ?Sized> (pub(crate) State<'a, T>);

pub struct State<'a, T: ?Sized> {
    strong: UnsafeCell<Vec<Strong<'a, T>>>,
    weak: UnsafeCell<Vec<Weak<'a, T>>>,
    inner: UnsafeCell<T>,
}

pub trait Listener<T: ?Sized> {
    fn receive(&self, v: &T);
}

impl<'a, T: ?Sized> State<'a, T> {
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

impl<'a, T: ?Sized> ReadState<'a, T> {
    #[inline]
    pub fn new(v: T) -> Self
    where
        T: Sized,
    {
        Self(State::new(v))
    }

    #[inline]
    pub fn get (&self) -> T where T: Copy {
        unsafe { *self.0.inner.get() }
    }

    #[inline]
    pub fn get_clone (&self) -> T where T: Clone {
        unsafe { &*self.0.inner.get() }.clone()
    }

    #[inline]
    pub fn with<U, F: FnOnce(&T) -> U>(&self, f: F) -> U {
        unsafe { f(&*self.0.inner.get()) }
    }

    #[inline]
    pub fn map_into<U: 'a, F: 'a + FnMut(&T) -> U> (&self, state: &'a State<U>, mut f: F) {
        self.register(move |x| state.set(f(x)));
    }
    
    // TODO get rid of listener trait and use flags to determine weak listener release (specially for spans, that may take references from rcs)
    #[inline]
    pub fn map_shared<U: 'a, F: 'a + FnMut(&T) -> U> (&self, mut f: F) -> Rc<ReadState<'a, U>> {
        let state = Rc::new(ReadState::new(self.with(&mut f)));
        let register_state = Rc::downgrade(&state);
        self.register_weak(move |x| {
            if let Some(register) = register_state.upgrade() {
                register.0.set(f(x));
                return true
            }
            return false
        });

        return state
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

macro_rules! impl_assign {
    ($($trait:ident as $fn:ident),+) => {
        $(
            impl<T: ?Sized + $trait<Rhs>, Rhs> $trait<Rhs> for State<'_, T> {
                #[inline]
                fn $fn(&mut self, rhs: Rhs) {
                    self.update(|x| x.$fn(rhs))
                }
            }

            impl<'a, T: ?Sized> State<'a, T> {
                #[inline]
                pub fn $fn<Rhs> (&self, rhs: Rhs) where T: $trait<Rhs> {
                    self.update(|x| x.$fn(rhs))
                }
            }
        )+
    };
}

impl_assign! {
    AddAssign as add_assign,
    SubAssign as sub_assign,
    MulAssign as mul_assign,
    DivAssign as div_assign,
    RemAssign as rem_assign,
    BitAndAssign as bitand_assign,
    BitOrAssign as bitor_assign,
    BitXorAssign as bitxor_assign
}

impl<'a, T> From<State<'a, T>> for ReadState<'a, T> {
    #[inline]
    fn from(value: State<'a, T>) -> Self {
        Self(value)
    }
}

impl<'a, T: ?Sized> Deref for State<'a, T> {
    type Target = ReadState<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*(self as *const Self as *const ReadState<'a, T>)
        }
    }
}

impl<T: ?Sized + Display> Display for State<'_, T> {
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