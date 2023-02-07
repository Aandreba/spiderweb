use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue, UnwrapThrowExt};

flat_mod! { node, text, element, component }
type DOMHighResTimeStamp = f64;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    type Window;
    #[derive(Debug, Clone)]
    #[wasm_bindgen(extends = DomNode)]
    type Document;
    #[derive(Debug, Clone)]
    #[wasm_bindgen(extends = DomNode)]
    pub(crate) type HtmlElement;
    type Performance;

    #[wasm_bindgen(structural, method, getter, js_class = "Window", js_name = document)]
    fn document(this: &Window) -> Option<Document>;
    #[wasm_bindgen (structural, method, getter, js_class = "Document", js_name = body)]
    fn body(this: &Document) -> Option<HtmlElement>;
    #[wasm_bindgen (structural, method, getter, js_class = "Window", js_name = performance)]
    fn performance (this: &Window) -> Option<Performance>;
    #[wasm_bindgen (structural, method)]
    fn create_element (this: &Document, tag: &str) -> HtmlElement;
    #[wasm_bindgen(structural, method)]
    fn now (this: &Performance) -> DOMHighResTimeStamp;
}

thread_local! {
    static WINDOW: Window = js_sys::global().dyn_into().expect_throw("Window not found");
    static DOCUMENT: Document = WINDOW.with(|win| win.document().expect_throw("Document not found!"));
    static BODY: HtmlElement = DOCUMENT.with(|doc| doc.body().expect_throw("Document not found!"));
    static PERFORMANCE: Performance = WINDOW.with(|win| win.performance().expect_throw("Performance API not detected"));
}

#[inline]
pub fn append_to_body<T: IntoNode> (node: T) -> Result<(), JsValue> {
    let node = node.into_node();
    BODY.with(|body| node.append_to(body))
}

#[inline]
pub(crate) fn create_element (tag: &str) -> HtmlElement {
    DOCUMENT.with(|doc| doc.create_element(tag))
}

#[inline]
pub(crate) fn now () -> DOMHighResTimeStamp {
    PERFORMANCE.with(|perf| perf.now())
}