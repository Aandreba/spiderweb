use crate::{
    dom::{ChildHandle, Component, DomHtmlElement, Element, IntoComponent},
    flag::{flag, Sender},
    state::StateCell,
};
use std::{
    any::Any,
    ops::{AddAssign, Deref, SubAssign},
    pin::Pin,
    ptr::NonNull,
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};

use super::{Orientation, Alignment};

pub type PaneChildHandleRef<'a> = PaneChildHandle<&'a Pane>;
pub type PaneChildHandleShared = PaneChildHandle<Rc<Pane>>;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct ParentPtr(NonNull<Element<StateCell<'static, f32>>>);

pub struct PaneChildHandle<P> {
    child: ChildHandle<ParentPtr>,
    parent: Pin<P>,
    size: f32,
    send: Sender,
}

pub struct Pane {
    inner: Element<StateCell<'static, f32>>,
    orientation: Orientation,
}

impl Pane {
    pub fn new(orient: Orientation, hoz: Alignment, vert: Alignment) -> Result<Self, JsValue> {
        let inner = Element::new("div", StateCell::new(0.0));

        let style = unsafe { inner.inner() }
            .unchecked_ref::<DomHtmlElement>()
            .style();
        style.set_property("display", "flex")?;
        style.set_property("flex-direction", orient.to_str())?;
        style.set_property("justify-content", hoz.to_str())?;
        style.set_property("align-items", vert.to_str())?;

        return Ok(Self {
            inner,
            orientation: orient,
        });
    }

    #[inline]
    pub fn horizontal(hoz: Alignment, vert: Alignment) -> Result<Self, JsValue> {
        Self::new(Orientation::Horizontal, hoz, vert)
    }

    #[inline]
    pub fn vertical(hoz: Alignment, vert: Alignment) -> Result<Self, JsValue> {
        Self::new(Orientation::Vertical, hoz, vert)
    }

    #[inline]
    pub fn push<T: IntoComponent>(
        &self,
        child: T,
        size: f32,
    ) -> Result<PaneChildHandleRef<'_>, JsValue> {
        Self::push_by_deref(self, child, size)
    }

    #[inline]
    pub fn push_shared<T: IntoComponent>(
        self: Rc<Self>,
        child: T,
        size: f32,
    ) -> Result<PaneChildHandleShared, JsValue> {
        Self::push_by_deref(self, child, size)
    }

    pub fn push_by_deref<D: Deref<Target = Self>, T: IntoComponent>(
        this: D,
        child: T,
        size: f32,
    ) -> Result<PaneChildHandle<D>, JsValue> {
        let child = child.into_component().render()?;
        let (send, recv) = flag();

        let style = unsafe { child.inner().style() };
        match this.orientation {
            Orientation::Horizontal => {
                style.set_property("height", "100%")?;
                this.inner.state().register_weak(move |sum| {
                    if recv.has_receiver() {
                        return false;
                    }
                    if let Err(e) = style.set_property("width", &format!("{}%", 100. * size / sum))
                    {
                        crate::macros::eprintln!(&e)
                    }
                    return true;
                });
            }

            Orientation::Vertical => {
                style.set_property("width", "100%")?;
                this.inner.state().register_weak(move |sum| {
                    if recv.has_receiver() {
                        return false;
                    }
                    if let Err(e) = style.set_property("height", &format!("{}%", 100. * size / sum))
                    {
                        crate::macros::eprintln!(&e)
                    }
                    return true;
                });
            }
        }

        this.inner.state().update(|sum| sum.add_assign(size));
        let this = Pin::new(this);
        let child = Element::append_child_by_deref(ParentPtr(NonNull::from(&this.inner)), child)?;

        return Ok(PaneChildHandle {
            parent: this,
            child,
            size,
            send,
        });
    }
}

impl<P: Deref<Target = Pane>> PaneChildHandle<P> {
    /// Detaches the child from it's parent
    #[inline]
    pub fn detach(self) -> Element<Pin<Box<dyn Any>>> {
        let inner = self.child.detach();
        self.send.send();
        self.parent
            .inner
            .state()
            .update(|x| x.sub_assign(self.size));
        return inner;
    }
}

impl Component for Pane {
    type State = StateCell<'static, f32>;

    #[inline]
    fn render(self) -> Result<crate::dom::Element<Self::State>, wasm_bindgen::JsValue> {
        Ok(self.inner)
    }
}

impl Deref for ParentPtr {
    type Target = Element<StateCell<'static, f32>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}
