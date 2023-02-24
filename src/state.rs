use std::{cell::UnsafeCell, ops::*, rc::Rc};

struct Inner<T> {
    subs: Vec<Box<dyn FnMut(&T)>>,
    inner: T
}

pub struct Readable<T> {
    inner: UnsafeCell<Inner<T>>
}

pub struct Writeable<T> {
    inner: Readable<T>
}

impl<T> Readable<T> {
    #[inline]
    pub const fn new (t: T) -> Self {
        return Self {
            inner: UnsafeCell::new(Inner {
                subs: Vec::new(),
                inner: t
            })
        }
    }

    #[inline]
    pub fn get (&self) -> T where T: Copy {
        unsafe { (&*self.inner.get()).inner }
    }

    #[inline]
    pub fn with<U, F: FnOnce(&T) -> U> (&self, f: F) -> U {
        unsafe { f(&(&*self.inner.get()).inner) }
    }

    #[inline]
    pub fn subscribe<F: 'static + FnMut(&T)> (&self, f: F) {
        Self::subscribe_by_deref(self, f)
    }

    #[inline]
    pub fn subscribe_boxed (&self, f: Box<dyn FnMut(&T)>) {
        Self::subscribe_boxed_by_deref(self, f)
    }

    #[inline]
    pub fn subscribe_shared<F: 'static + FnMut(&T)> (self: Rc<Self>, f: F) {
        Self::subscribe_by_deref(self, f)
    }

    #[inline]
    pub fn subscribe_shared_boxed (self: Rc<Self>, f: Box<dyn FnMut(&T)>) {
        Self::subscribe_boxed_by_deref(self, f)
    }

    #[inline]
    pub fn subscribe_by_deref<D: Deref<Target = Self>, F: 'static + FnMut(&T)> (this: D, f: F) {
        Self::subscribe_boxed_by_deref(this, Box::new(f))
    }
    
    #[inline]
    pub fn subscribe_boxed_by_deref<D: Deref<Target = Self>> (this: D, f: Box<dyn FnMut(&T)>){
        unsafe { &mut *this.inner.get() }.subs.push(f)
    }
}

impl<T> Writeable<T> {
    #[inline]
    pub const fn new (t: T) -> Self {
        return Self {
            inner: Readable::new(t)
        }
    }

    #[inline]
    pub fn set (&self, v: T) {
        let inner = unsafe { &mut *self.inner.inner.get() };
        inner.inner = v;
        self.notify()
    }

    #[inline]
    pub fn replace (&self, v: T) -> T {
        let inner = unsafe { &mut *self.inner.inner.get() };
        let prev = core::mem::replace(&mut inner.inner, v);
        self.notify();
        return prev
    }

    #[inline]
    pub fn take (&self) -> T where T: Default {
        self.replace(Default::default())
    }

    #[inline]
    pub fn update<U, F: FnOnce(&mut T) -> U> (&self, f: F) -> U {
        let inner = unsafe { &mut *self.inner.inner.get() };
        let res = f(&mut inner.inner);
        self.notify();
        return res
    }

    #[inline]
    fn notify (&self) {
        let inner = unsafe { &mut *self.inner.inner.get() };
        for sub in inner.subs.iter_mut() {
            sub(&inner.inner)
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