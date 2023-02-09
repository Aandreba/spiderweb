use std::{rc::{Rc, Weak}, cell::Cell};

#[inline]
pub fn flag() -> (Sender, Receiver) {
    let inner = Rc::new(Cell::new(false));
    return (
        Sender {
            inner: Rc::downgrade(&inner),
        },
        Receiver { inner },
    );
}

#[derive(Debug, Clone)]
pub struct Sender {
    inner: Weak<Cell<bool>>,
}

#[derive(Debug, Clone)]
pub struct Receiver {
    inner: Rc<Cell<bool>>,
}

impl Sender {
    #[inline]
    pub fn send (&self) {
        let _ = self.try_send();
    }

    pub fn try_send (&self) -> bool {
        if let Some(inner) = self.inner.upgrade() {
            inner.set(true);
            return true
        }
        return false
    }
}

impl Receiver {
    #[inline]
    pub fn has_receiver (&self) -> bool {
        self.inner.get()
    }
}