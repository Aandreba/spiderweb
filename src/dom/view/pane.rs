use crate::{
    dom::{ChildHandle, Component, DomHtmlElement, Element, IntoComponent},
    flag::{flag, Sender},
    state::{StateCell, ReadOnlyState},
};
use std::{
    ops::{AddAssign, Deref, SubAssign},
    pin::Pin,
    ptr::NonNull,
    rc::Rc, any::Any,
};
use wasm_bindgen::{JsCast, JsValue};
use super::{Orientation, Alignment};

pub type PaneChildHandleRef<'a, T> = PaneChildHandle<T, &'a Pane>;
pub type PaneChildHandleShared<T> = PaneChildHandle<T, Rc<Pane>>;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct ParentPtr(NonNull<Element<StateCell<'static, f32>>>);

pub struct PaneChildHandle<T, P> {
    child: ChildHandle<T, ParentPtr>,
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
    ) -> Result<PaneChildHandleRef<'_, T::State>, JsValue> {
        self.push_weighted(child, 1.)
    }

    #[inline]
    pub fn push_weighted<T: IntoComponent>(
        &self,
        child: T,
        size: f32,
    ) -> Result<PaneChildHandleRef<'_, T::State>, JsValue> {
        Self::push_by_deref(self, child, size)
    }

    #[inline]
    pub fn push_shared<T: IntoComponent>(
        self: Rc<Self>,
        child: T,
    ) -> Result<PaneChildHandleShared<T::State>, JsValue> {
        self.push_weighted_shared(child, 1.)
    }

    #[inline]
    pub fn push_weighted_shared<T: IntoComponent>(
        self: Rc<Self>,
        child: T,
        size: f32,
    ) -> Result<PaneChildHandleShared<T::State>, JsValue> {
        Self::push_by_deref(self, child, size)
    }

    pub fn push_by_deref<D: Deref<Target = Self>, T: IntoComponent>(
        this: D,
        child: T,
        size: f32,
    ) -> Result<PaneChildHandle<T::State, D>, JsValue> {
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
                        #[cfg(debug_assertions)]
                        crate::eprintln!(&e)
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
                        #[cfg(debug_assertions)]
                        crate::eprintln!(&e)
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

impl<T: Any, P: Deref<Target = Pane>> PaneChildHandle<T, P> {
    /// Returns a reference to the child's state
    #[inline]
    pub fn state (&self) -> &T {
        self.child.state()
    }

    /// Detaches the child from it's parent
    #[inline]
    pub fn detach(self) -> Element<T> {
        let inner = self.child.detach();
        self.send.send();
        self.parent
            .inner
            .state()
            .update(|x| x.sub_assign(self.size));
        return inner;
    }
}

impl<T, P: Deref<Target = Pane>> Deref for PaneChildHandle<T, P> {
    type Target = Element<Box<dyn Any>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.child.deref()
    }
}

impl Component for Pane {
    type State = ReadOnlyState<'static, f32>;

    #[inline]
    fn render(self) -> Result<crate::dom::Element<Self::State>, wasm_bindgen::JsValue> {
        Ok(Element {
            inner: self.inner.inner,
            current_id: self.inner.current_id,
            children: self.inner.children,
            state: self.inner.state.into(),
        })
    }
}

impl Deref for ParentPtr {
    type Target = Element<StateCell<'static, f32>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}
