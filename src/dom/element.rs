use js_sys::Function;
use slab::Slab;
use std::{cell::UnsafeCell, ops::Deref, any::Any, pin::Pin};
use wasm_bindgen::{prelude::{wasm_bindgen, Closure}, JsCast, JsValue};

use super::component::Component;

thread_local! {
    pub static DOCUMENT: Document = document();
    pub static BODY: Element = Element {
        inner: UnsafeCell::new(Inner {
            element: DOCUMENT.with(Document::body),
            children: Slab::new(),
            listeners: Slab::new()
        })
    };
}

#[wasm_bindgen]
extern "C" {
    pub type Document;
    #[wasm_bindgen(extends = Node, js_name = HTMLElement)]
    type HtmlElement;
    #[wasm_bindgen(extends = EventTarget)]
    type Node;
    type EventTarget;

    fn document() -> Document;
    #[wasm_bindgen(structural, method)]
    fn body(this: &Document) -> HtmlElement;
    #[wasm_bindgen(structural, method, js_name = createElement)]
    fn create_element(this: &Document, tag: &str) -> HtmlElement;

    #[wasm_bindgen(structural, method, catch, js_name = appendChild)]
    fn append_child (this: &Node, child: &Node) -> Result<Node, JsValue>;
    #[wasm_bindgen(structural, method, catch, js_name = removeChild)]
    fn remove_child (this: &Node, child: &Node) -> Result<Node, JsValue>;

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

struct Inner {
    element: HtmlElement,
    children: Slab<Element>,
    listeners: Slab<(&'static str, Closure<dyn FnMut()>)>,
}

pub struct Element {
    inner: UnsafeCell<Inner>,
}

pub struct ElementRef<'a> {
    parent: &'a Element,
    idx: usize,
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
        };

        return Self {
            inner: UnsafeCell::new(inner),
        };
    }

    #[inline]
    pub fn append_child(&self, element: impl Into<Child>) -> Result<ElementRef<'_>, JsValue> {
        let this = unsafe { &mut *self.inner.get() };
        let element: Child = element.into();

        this.element.append_child(&unsafe { &*element.inner.get() }.element)?;

        let idx = this
            .children
            .insert(element);
        return Ok(ElementRef { parent: self, idx });
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

impl<'a> Deref for ElementRef<'a> {
    type Target = Element;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            return (&*self.parent.inner.get())
                .children
                .get(self.idx)
                .unwrap_unchecked();
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
    fn element (&self) -> &HtmlElement {
        unsafe {
            match self {
                Self::Element(x) => &(&*x.inner.get()).element,
                Self::Component(x) => &(&*x.inner.inner.get()).element
            }
        }
    }
}

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