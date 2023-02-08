use std::{ops::{AddAssign, Deref, SubAssign}, pin::Pin, ptr::NonNull, any::Any, rc::Rc};
use wasm_bindgen::{JsValue, JsCast, prelude::wasm_bindgen};
use crate::{dom::{IntoComponent, Component, Element, DomHtmlElement, ChildHandle}, state::{StateCell}, channel::oneshot::{self, Sender}};

pub type PaneChildHandleRef<'a> = PaneChildHandle<&'a Pane>;
pub type PaneChildHandleShared = PaneChildHandle<Rc<Pane>>;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct ParentPtr (NonNull<Element<StateCell<'static, f32>>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    Vertical,
    Horizontal
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alignment {
    Start = "flex-start",
    Center = "center",
    End = "flex-end"
}

pub struct PaneChildHandle<P> {
    parent: Pin<P>,
    child: ChildHandle<ParentPtr>,
    size: f32,
    send: Sender<()>
}

pub struct Pane {
    inner: Element<StateCell<'static, f32>>,
    orientation: Orientation
}

impl Pane {
    pub fn new (orient: Orientation, hoz: Alignment, vert: Alignment) -> Result<Self, JsValue> {
        let inner = Element::new("div", StateCell::new(0.0));

        let style = inner.inner.unchecked_ref::<DomHtmlElement>().style();
        style.set_property("display", "flex")?;
        style.set_property("justify-content", hoz.to_str())?;
        style.set_property("align-items", vert.to_str())?;
        
        return Ok(Self {
            inner,
            orientation: orient,
        })
    }

    #[inline]
    pub fn push<T: IntoComponent> (&self, child: T, size: f32) -> Result<PaneChildHandleRef<'_>, JsValue> {
        Self::push_by_deref(self, child, size)
    }

    #[inline]
    pub fn push_shared<T: IntoComponent> (self: Rc<Self>, child: T, size: f32) -> Result<PaneChildHandleShared, JsValue> {
        Self::push_by_deref(self, child, size)
    }

    pub fn push_by_deref<D: Deref<Target = Self>, T: IntoComponent> (this: D, child: T, size: f32) -> Result<PaneChildHandle<D>, JsValue> {
        let child = child.into_component().render()?;
        let (send, recv) = oneshot::channel::<()>();

        if let Some(child) = child.inner.dyn_ref::<DomHtmlElement>() {
            let style = child.style();

            match this.orientation {
                Orientation::Horizontal => {
                    style.set_property("height", "100%")?;
                    this.inner.state().register_weak(move |sum| {
                        if recv.is_available() { return false }
                        if let Err(e) = style.set_property("width", &format!("calc(100% * {} / {})", size, sum)) {
                            crate::macros::eprintln!(&e)
                        }
                        return true
                    });
                },

                Orientation::Vertical => {
                    style.set_property("width", "100%")?;
                    this.inner.state().register_weak(move |sum| {
                        if recv.is_available() { return false }
                        if let Err(e) = style.set_property("height", &format!("calc(100% * {} / {})", size, sum)) {
                            crate::macros::eprintln!(&e)
                        }
                        return true
                    });
                }
            }
        }

        this.inner.state().update(|sum| sum.add_assign(size));
        let this = Pin::new(this);
        let child = Element::append_child_by_deref(ParentPtr(NonNull::from(&this.inner)), child)?;

        return Ok(PaneChildHandle {
            parent: this,
            child,
            size,
            send
        })
    }
}

impl<P: Deref<Target = Pane>> PaneChildHandle<P> {
    #[inline]
    pub fn detach (self) -> Element<Pin<Box<dyn Any>>> {
        let inner = self.child.detach();
        self.send.send(());
        self.parent.inner.state().update(|x| x.sub_assign(self.size));
        return inner
    }
}

impl Component for Pane {
    type State = StateCell<'static, f32>;

    #[inline]
    fn render (self) -> Result<crate::dom::Element<Self::State>, wasm_bindgen::JsValue> {
        Ok(self.inner)
    }
}

impl Default for Alignment {
    #[inline]
    fn default() -> Self {
        Alignment::Center
    }
}

impl Deref for ParentPtr {
    type Target = Element<StateCell<'static, f32>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}