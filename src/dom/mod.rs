use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue, UnwrapThrowExt};
flat_mod! { node, text }

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    type Window;
    #[derive(Debug, Clone)]
    type Document;

    #[derive(Debug, Clone)]
    #[wasm_bindgen(extends = JsNode)]
    type HtmlElement;

    #[wasm_bindgen(structural, method, getter, js_class = "Window", js_name = document)]
    fn document(this: &Window) -> Option<Document>;
    #[wasm_bindgen (structural, method, getter, js_class = "Document", js_name = body)]
    fn body(this: &Document) -> Option<HtmlElement>;

}

thread_local! {
    static WINDOW: Window = js_sys::global().dyn_into().expect_throw("Window not found");
    static DOCUMENT: Document = WINDOW.with(|win| win.document().expect_throw("Document not found!"));
    static BODY: HtmlElement = DOCUMENT.with(|doc| doc.body().expect_throw("Document not found!"));
}

#[inline]
pub fn append_to_body<T: Node>(node: T) -> Result<(), JsValue> {
    BODY.with(|body| T::append_to(node, body))
}
