use std::{cell::UnsafeCell, ops::*, rc::Rc};

pub struct Readable<T: ?Sized> {
    strong: UnsafeCell<Vec<Box<dyn FnMut(&T)>>>,
    weak: UnsafeCell<Vec<Box<dyn FnMut(&T) -> bool>>>,
    inner: UnsafeCell<T>,
}

#[repr(transparent)]
pub struct Writeable<T: ?Sized> {
    inner: Readable<T>,
}

impl<T: ?Sized> Readable<T> {
    #[inline]
    pub const fn new(t: T) -> Self where T: Sized {
        return Self {
            strong: UnsafeCell::new(Vec::new()),
            weak: UnsafeCell::new(Vec::new()),
            inner: UnsafeCell::new(t),
        };
    }

    #[inline]
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe { *self.inner.get() }
    }

    #[inline]
    pub fn with<U, F: FnOnce(&T) -> U>(&self, f: F) -> U {
        unsafe { f(&*self.inner.get()) }
    }

    #[inline]
    pub fn map<U: 'static, F: 'static + FnMut(&T) -> U>(&self, mut f: F) -> Rc<Readable<U>> {
        let result = Rc::new(Writeable::new(self.with(&mut f)));
        let target = Rc::downgrade(&result);

        self.subscribe_weak(move |x| match target.upgrade() {
            Some(target) => {
                target.set(f(x));
                true
            }
            None => false,
        });

        return unsafe { Rc::from_raw(Rc::into_raw(result).cast()) };
    }

    #[inline]
    pub fn map_into<U: 'static, F: 'static + FnMut(&T) -> U>(
        &self,
        target: Writeable<U>,
        mut f: F,
    ) {
        self.subscribe(move |x| target.set(f(x)))
    }

    #[inline]
    pub fn subscribe<F: 'static + FnMut(&T)>(&self, f: F) {
        self.subscribe_boxed(Box::new(f))
    }

    #[inline]
    pub fn subscribe_weak<F: 'static + FnMut(&T) -> bool>(&self, f: F) {
        self.subscribe_weak_boxed(Box::new(f))
    }

    #[inline]
    pub fn subscribe_boxed(&self, f: Box<dyn FnMut(&T)>) {
        unsafe { &mut *self.strong.get() }.push(f)
    }

    #[inline]
    pub fn subscribe_weak_boxed(&self, f: Box<dyn FnMut(&T) -> bool>) {
        unsafe { &mut *self.weak.get() }.push(f)
    }
}

impl<T: ?Sized> Writeable<T> {
    #[inline]
    pub const fn new(t: T) -> Self where T: Sized {
        return Self {
            inner: Readable::new(t),
        };
    }

    #[inline]
    pub fn set(&self, v: T) where T: Sized {
        unsafe { *self.inner.inner.get() = v };
        self.notify()
    }

    #[inline]
    pub fn replace(&self, v: T) -> T where T: Sized {
        let prev = core::mem::replace(unsafe { &mut *self.inner.inner.get() }, v);
        self.notify();
        return prev;
    }

    #[inline]
    pub fn take(&self) -> T
    where
        T: Default,
    {
        self.replace(Default::default())
    }

    #[inline]
    pub fn update<U, F: FnOnce(&mut T) -> U>(&self, f: F) -> U {
        let res = f(unsafe { &mut *self.inner.inner.get() });
        self.notify();
        return res;
    }

    #[inline]
    fn notify(&self) {
        let inner = unsafe { &*self.inner.inner.get() };
        let strong = unsafe { &mut *self.inner.strong.get() };
        let weak = unsafe { &mut *self.inner.weak.get() };

        for sub in strong.iter_mut() {
            sub(inner)
        }

        let mut i = 0;
        while i < weak.len() {
            if !unsafe { weak.get_unchecked_mut(i) }(inner) {
                weak.swap_remove(i);
            } else {
                i += 1
            }
        }
    }
}

impl<T> Deref for Writeable<T> {
    type Target = Readable<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

macro_rules! impl_assign {
    ($($trait:ident as $f:ident),+) => {
        $(
            impl<T: $trait<U>, U> $trait<U> for Writeable<T> {
                #[inline]
                fn $f (&mut self, rhs: U) {
                    self.update(|x| x.$f(rhs))
                }
            }

            impl<T> Writeable<T> {
                #[inline]
                pub fn $f<U> (&self, rhs: U) where T: $trait<U> {
                    self.update(|x| x.$f(rhs))
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
    BitXorAssign as bitxor_assign,
    ShlAssign as shl_assign,
    ShrAssign as shr_assign
}
