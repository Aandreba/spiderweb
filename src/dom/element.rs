use js_sys::Function;
use slab::Slab;
use std::{cell::UnsafeCell, ops::Deref, any::Any, pin::Pin, rc::Rc};
use wasm_bindgen::{prelude::{wasm_bindgen, Closure}, JsCast, JsValue};
use crate::state::Readable;

use super::component::{Component, MountedComponent};

thread_local! {
    pub static DOCUMENT: Document = window().document();
    pub static BODY: Rc<Element> = Rc::new(Element {
        inner: UnsafeCell::new(Inner {
            element: DOCUMENT.with(Document::body),
            children: Slab::new(),
            listeners: Slab::new(),
            texts: Slab::new()
        })
    });
}

#[wasm_bindgen]
extern "C" {
    pub type Window;
    pub type Document;
    
    #[wasm_bindgen(extends = Node, js_name = HTMLElement)]
    pub(super) type HtmlElement;

    #[derive(Clone)]
    #[wasm_bindgen(extends = Node)]
    type Text;

    #[derive(Clone)]
    #[wasm_bindgen(extends = EventTarget)]
    pub(super) type Node;

    #[derive(Clone)]
    pub(super) type EventTarget;

    #[wasm_bindgen(structural, method, getter)]
    fn document(this: &Window) -> Document;

    #[wasm_bindgen(structural, method, getter)]
    fn body(this: &Document) -> HtmlElement;
    #[wasm_bindgen(structural, method, js_name = createElement)]
    fn create_element(this: &Document, tag: &str) -> HtmlElement;

    #[wasm_bindgen(structural, method, catch, js_name = appendChild)]
    fn append_child (this: &Node, child: &Node) -> Result<Node, JsValue>;
    #[wasm_bindgen(structural, method, catch, js_name = removeChild)]
    fn remove_child (this: &Node, child: &Node) -> Result<Node, JsValue>;

    #[wasm_bindgen(constructor)]
    fn new (s: &str) -> Text;
    #[wasm_bindgen(structural, method, getter)]
    fn data (this: &Text) -> String;
    #[wasm_bindgen(structural, method, setter, js_name = data)]
    fn set_data (this: &Text, s: &str);

    #[wasm_bindgen(structural, method, js_name = addEventListener)]
    fn add_event_listener (this: &EventTarget, event: &str, f: &Function);
    #[wasm_bindgen(structural, method, js_name = removeEventListener)]
    fn remove_event_listener (this: &EventTarget, event: &str, f: &Function);
}

#[doc(hidden)]
pub enum Child {
    Element (Element),
    Component (Pin<Box<Component<dyn Any>>>)
}

enum InnerText {
    Owned (Box<Readable<dyn AsRef<str>>>),
    Shared (Rc<Readable<dyn AsRef<str>>>)
}

pub(super) struct Inner {
    pub(super) element: HtmlElement,
    pub(super) children: Slab<Child>,
    pub(super) listeners: Slab<(&'static str, Closure<dyn FnMut()>)>,
    pub(super) texts: Slab<InnerText>
}

pub struct Element {
    pub(super) inner: UnsafeCell<Inner>,
}

pub struct MountedElement<P> {
    pub(super) parent: P,
    pub(super) idx: usize,
}

pub struct ListenerRef<'a> {
    parent: &'a Element,
    idx: usize,
}

impl Element {
    #[inline]
    pub fn new(tag: &str) -> Self {
        let inner = Inner {
            element: DOCUMENT.with(|doc| doc.create_element(tag)),
            children: Slab::new(),
            listeners: Slab::new(),
            texts: Slab::new(),
        };

        return Self {
            inner: UnsafeCell::new(inner),
        };
    }

    #[inline]
    pub fn add_text (&self, s: &str) -> Result<(), JsValue> {
        let text: Text = Text::new(s);
        unsafe { &*self.inner.get() }.element.append_child(&text)?;
        return Ok(())
    }

    #[inline]
    pub fn bind_text<T: AsRef<str>> (&self, state: Readable<T>) -> Result<(), JsValue> {
        let state = &state as &Readable<dyn AsRef<str>>;
        let text: Text = state.with(|x| Text::new(x.as_ref()));

        let my_text = text.clone();
        state.subscribe(move |x| my_text.set_data(x.as_ref())); // this currently leaks!

        unsafe { &*self.inner.get() }.element.append_child(&text)?;
        return Ok(())
    }

    #[inline]
    pub fn create_component<'a, T: Any> (&'a self, tag: &str, state: T) -> Result<Pin<MountedComponent<&'a Self, T>>, JsValue> {
        Self::create_component_by_deref(self, tag, state)
    }

    #[inline]
    pub fn create_component_shared<T: Any> (self: Rc<Self>, tag: &str, state: T) -> Result<Pin<MountedComponent<Rc<Self>, T>>, JsValue> {
        Self::create_component_by_deref(self, tag, state)
    }

    #[inline]
    pub fn create_component_by_deref<'a, D: 'a + Deref<Target = Self>, T: Any> (this: D, tag: &str, state: T) -> Result<Pin<MountedComponent<'a, D, T>>, JsValue> {
        let comp = Component::new(tag, state);
        let inner = Self::append_child_by_deref(this, comp)?;
        return unsafe { Ok(Pin::new_unchecked(MountedComponent { handle: inner, _phtm: std::marker::PhantomData })) }
    }

    #[inline]
    pub fn append_child (&self, element: impl Into<Child>) -> Result<MountedElement<&Self>, JsValue> {
        Self::append_child_by_deref(self, element)
    }

    #[inline]
    pub fn append_child_shared (self: Rc<Self>, element: impl Into<Child>) -> Result<MountedElement<Rc<Self>>, JsValue> {
        Self::append_child_by_deref(self, element)
    }

    #[inline]
    pub fn append_child_by_deref<D: Deref<Target = Self>> (this: D, element: impl Into<Child>) -> Result<MountedElement<D>, JsValue> {
        let inner = unsafe { &mut *this.inner.get() };
        let element: Child = element.into();
        inner.element.append_child(element.html_element())?;

        let idx = inner
            .children
            .insert(element);
        return Ok(MountedElement { parent: this, idx });
    }

    #[inline]
    pub fn add_event_listener<'a>(
        &'a self,
        event: &'static str,
        f: Box<dyn FnMut()>,
    ) -> ListenerRef<'a> {
        let this = unsafe { &mut *self.inner.get() };
        let f = Closure::wrap(f);

        this.element.add_event_listener(event, f.as_ref().unchecked_ref());
        let idx = this.listeners.insert((event, f));
        
        return ListenerRef {
            parent: self,
            idx,
        };
    }
}

impl<P: Deref<Target = Element>> Deref for MountedElement<P> {
    type Target = Element;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            return (&*self.parent.inner.get())
                .children
                .get(self.idx)
                .unwrap_unchecked()
                .element();
        }
    }
}

impl Drop for Element {
    #[inline]
    fn drop(&mut self) {
        let inner = self.inner.get_mut();
        for (event, f) in inner.listeners.drain() {
            inner.element.remove_event_listener(event, f.as_ref().unchecked_ref());
        }
    }
}

impl Child {
    #[inline]
    fn element (&self) -> &Element {
        match self {
            Self::Element(x) => x,
            Self::Component(x) => &x.element
        }
    }

    #[inline]
    fn html_element (&self) -> &HtmlElement {
        &unsafe { &*self.element().inner.get() }.element
    }
}

/* CHILD */
impl From<Element> for Child {
    #[inline]
    fn from(value: Element) -> Self {
        Self::Element(value)
    }
}

impl<T: Any> From<Component<T>> for Child {
    #[inline]
    fn from(value: Component<T>) -> Self {
        Self::Component(Box::pin(value))
    }
}

impl From<Box<Component<dyn Any>>> for Child {
    #[inline]
    fn from(value: Box<Component<dyn Any>>) -> Self {
        Self::Component(Box::into_pin(value))
    }
}

impl From<Pin<Box<Component<dyn Any>>>> for Child {
    #[inline]
    fn from(value: Pin<Box<Component<dyn Any>>>) -> Self {
        Self::Component(value)
    }
}

#[inline]
pub fn window () -> Window {
    use crate::wasm_bindgen::UnwrapThrowExt;
    JsCast::dyn_into(js_sys::global()).unwrap_throw()
}

#[inline]
pub fn body () -> Rc<Element> {
    BODY.with(Clone::clone)
}